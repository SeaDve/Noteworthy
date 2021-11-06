mod repository;
mod repository_watcher;
mod sync_state;

use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::{sync::Lazy, unsync::OnceCell};
use regex::Regex;

use std::{
    cell::Cell,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub use self::sync_state::SyncState;
use self::{repository::Repository, repository_watcher::RepositoryWatcher};
use crate::{spawn, spawn_blocking};

const DEFAULT_REMOTE_NAME: &str = "origin";
const DEFAULT_AUTHOR_NAME: &str = "NoteworthyApp";
const DEFAULT_AUTHOR_EMAIL: &str = "app@noteworthy.io";

static RE_VALIDATE_URL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(git@[\w\.]+)(:(//)?)([\w\.@:/\-~]+)(\.git)(/)?").unwrap());

struct SyncOptions {
    is_skip_pull: bool,
    is_skip_push: bool,
}

mod imp {
    use super::*;

    #[derive(Default, Debug)]
    pub struct NoteRepository {
        pub base_path: OnceCell<gio::File>,
        pub sync_state: Cell<SyncState>,
        pub repository: OnceCell<Arc<Mutex<Repository>>>,
        pub watcher: OnceCell<RepositoryWatcher>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NoteRepository {
        const NAME: &'static str = "NwtyNoteRepository";
        type Type = super::NoteRepository;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for NoteRepository {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "base-path",
                        "Base Path",
                        "Where the repository is stored locally",
                        gio::File::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_enum(
                        "sync-state",
                        "Sync State",
                        "Current sync state",
                        SyncState::static_type(),
                        SyncState::default() as i32,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "base-path" => {
                    let base_path = value.get().unwrap();
                    self.base_path.set(base_path).unwrap();
                }
                "sync-state" => {
                    let sync_state = value.get().unwrap();
                    obj.set_sync_state(sync_state);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "base-path" => obj.base_path().to_value(),
                "sync-state" => obj.sync_state().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct NoteRepository(ObjectSubclass<imp::NoteRepository>);
}

impl NoteRepository {
    pub async fn init(base_path: &gio::File) -> anyhow::Result<Self> {
        let repository_path = base_path.path().unwrap();
        let repository = spawn_blocking!(move || Repository::init(&repository_path)).await?;
        Ok(Self::new(base_path, repository))
    }

    pub async fn clone(remote_url: String, base_path: &gio::File) -> anyhow::Result<Self> {
        let repository_path = base_path.path().unwrap();
        let repository =
            spawn_blocking!(move || Repository::clone(&repository_path, &remote_url)).await?;
        Ok(Self::new(base_path, repository))
    }

    pub async fn open(base_path: &gio::File) -> anyhow::Result<Self> {
        let repository_path = base_path.path().unwrap();
        let repository = spawn_blocking!(move || Repository::open(&repository_path)).await?;
        Ok(Self::new(base_path, repository))
    }

    fn new(base_path: &gio::File, repository: Repository) -> Self {
        let obj = glib::Object::new::<Self>(&[("base-path", &base_path)])
            .expect("Failed to create NoteRepository.");
        obj.set_repository(repository);
        obj
    }

    pub fn validate_remote_url(remote_url: &str) -> bool {
        if remote_url.is_empty() {
            return false;
        }

        RE_VALIDATE_URL.is_match(remote_url)
    }

    pub fn sync_state(&self) -> SyncState {
        let imp = imp::NoteRepository::from_instance(self);
        imp.sync_state.get()
    }

    pub fn connect_remote_changed<F: Fn(&RepositoryWatcher) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        spawn!(clone!(@weak self as obj => async move {
            assert!(!obj.is_offline_mode().await, "Trying to connect remote change even it is offline mode");
        }));

        let base_path = self.base_path();
        let watcher = RepositoryWatcher::new(&base_path, DEFAULT_REMOTE_NAME);
        let handler_id = watcher.connect_remote_changed(f);

        // Store a strong reference
        let imp = imp::NoteRepository::from_instance(self);
        imp.watcher.set(watcher).unwrap();

        handler_id
    }

    pub async fn sync(&self) -> anyhow::Result<Vec<(PathBuf, git2::Delta)>> {
        let sync_opts = SyncOptions {
            is_skip_pull: false,
            is_skip_push: false,
        };

        let changed_files = self.sync_full(sync_opts).await?.unwrap();
        Ok(changed_files)
    }

    pub async fn sync_offline(&self) -> anyhow::Result<()> {
        let sync_opts = SyncOptions {
            is_skip_pull: true,
            is_skip_push: true,
        };

        match self.sync_full(sync_opts).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    async fn sync_full(
        &self,
        sync_opts: SyncOptions,
    ) -> anyhow::Result<Option<Vec<(PathBuf, git2::Delta)>>> {
        self.set_sync_state(SyncState::Syncing);

        let changed_files = if !sync_opts.is_skip_pull {
            log::info!("Sync: Repo pulling changes...");
            self.set_sync_state(SyncState::Pulling);
            let changed_files = self.pull().await?;
            log::info!("Sync: Repo pulled changes");
            Some(changed_files)
        } else {
            None
        };

        if self.is_file_changed_in_workdir().await? {
            log::info!("Sync: Found changes, adding all...");
            self.add_all().await?;
            log::info!("Sync: Added all files");

            log::info!("Sync: Creating commit...");
            self.commit().await?;
            log::info!("Sync: Created commit");

            if !sync_opts.is_skip_push {
                log::info!("Sync: Repo pushing changes...");
                self.set_sync_state(SyncState::Pushing);
                self.push().await?;
                log::info!("Sync: Pushed chanes to remote");
            }
        } else {
            log::info!("Sync: There is no changed files in directory");
            log::info!("Sync: Skipped pushing and commit");
        }

        self.set_sync_state(SyncState::Idle);

        Ok(changed_files)
    }

    // FIXME (CRITICAL) handle conflicts gracefully
    async fn pull(&self) -> anyhow::Result<Vec<(PathBuf, git2::Delta)>> {
        let repo = self.repository();

        spawn_blocking!(move || {
            let repo = repo.lock().unwrap();

            repo.pull(
                DEFAULT_REMOTE_NAME,
                DEFAULT_AUTHOR_NAME,
                DEFAULT_AUTHOR_EMAIL,
            )
        })
        .await
    }

    async fn remotes(&self) -> anyhow::Result<Vec<String>> {
        let repo = self.repository();

        spawn_blocking!(move || {
            let repo = repo.lock().unwrap();

            repo.remotes()
        })
        .await
    }

    async fn is_offline_mode(&self) -> bool {
        self.remotes().await.map_or(true, |r| r.is_empty())
    }

    async fn is_file_changed_in_workdir(&self) -> anyhow::Result<bool> {
        let repo = self.repository();

        spawn_blocking!(move || {
            let repo = repo.lock().unwrap();

            repo.is_file_changed_in_workdir()
        })
        .await
    }

    async fn add_all(&self) -> anyhow::Result<()> {
        let repo = self.repository();

        spawn_blocking!(move || {
            let repo = repo.lock().unwrap();

            repo.add(&["."])
        })
        .await
    }

    async fn commit(&self) -> anyhow::Result<()> {
        let repo = self.repository();

        spawn_blocking!(move || {
            let repo = repo.lock().unwrap();

            repo.commit("Sync commit", DEFAULT_AUTHOR_NAME, DEFAULT_AUTHOR_EMAIL)
        })
        .await
    }

    async fn push(&self) -> anyhow::Result<()> {
        let repo = self.repository();

        spawn_blocking!(move || {
            let repo = repo.lock().unwrap();

            repo.push(DEFAULT_REMOTE_NAME)
        })
        .await
    }

    fn repository(&self) -> Arc<Mutex<Repository>> {
        let imp = imp::NoteRepository::from_instance(self);
        Arc::clone(imp.repository.get().unwrap())
    }

    fn set_repository(&self, repository: Repository) {
        let imp = imp::NoteRepository::from_instance(self);
        imp.repository
            .set(Arc::new(Mutex::new(repository)))
            .unwrap();
    }

    fn base_path(&self) -> gio::File {
        let imp = imp::NoteRepository::from_instance(self);
        imp.base_path.get().unwrap().clone()
    }

    fn set_sync_state(&self, sync_state: SyncState) {
        let imp = imp::NoteRepository::from_instance(self);
        imp.sync_state.set(sync_state);
        self.notify("sync-state");
    }
}

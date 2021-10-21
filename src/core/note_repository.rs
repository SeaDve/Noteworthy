use gtk::{
    gio,
    glib::{self, GEnum},
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

use super::{repository::Repository, repository_watcher::RepositoryWatcher};
use crate::utils;

const DEFAULT_REMOTE_NAME: &str = "origin";
const DEFAULT_AUTHOR_NAME: &str = "NoteworthyApp";
const DEFAULT_AUTHOR_EMAIL: &str = "app@noteworthy.io";

static RE_VALIDATE_URL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(git@[\w\.]+)(:(//)?)([\w\.@:/\-~]+)(\.git)(/)?").unwrap());

#[derive(Clone, Copy, Debug, PartialEq, GEnum)]
#[genum(type_name = "NwtyNoteRepositorySyncState")]
pub enum SyncState {
    Idle,
    Pulling,
    Pushing,
}

impl Default for SyncState {
    fn default() -> Self {
        Self::Idle
    }
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

        fn constructed(&self, obj: &Self::Type) {
            let base_path = obj.base_path();
            let watcher = RepositoryWatcher::new(&base_path, DEFAULT_REMOTE_NAME);
            self.watcher.set(watcher).unwrap();
        }
    }
}

glib::wrapper! {
    pub struct NoteRepository(ObjectSubclass<imp::NoteRepository>);
}

impl NoteRepository {
    pub async fn clone(remote_url: String, base_path: &gio::File) -> anyhow::Result<Self> {
        let repository_path = base_path.path().unwrap();
        let repository =
            utils::do_async(move || Repository::clone(&repository_path, &remote_url)).await?;
        let obj = glib::Object::new::<Self>(&[("base-path", &base_path)])
            .expect("Failed to create NoteRepository.");

        obj.set_repository(repository);
        Ok(obj)
    }

    pub async fn open(base_path: &gio::File) -> anyhow::Result<Self> {
        let repository_path = base_path.path().unwrap();
        let repository = utils::do_async(move || Repository::open(&repository_path)).await?;
        let obj = glib::Object::new::<Self>(&[("base-path", &base_path)])
            .expect("Failed to create NoteRepository.");

        obj.set_repository(repository);
        Ok(obj)
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
        let imp = imp::NoteRepository::from_instance(self);
        imp.watcher.get().unwrap().connect_remote_changed(f)
    }

    pub async fn sync(&self) -> anyhow::Result<Vec<(PathBuf, git2::Delta)>> {
        log::info!("Sync: Repo pulling changes...");
        self.set_sync_state(SyncState::Pulling);
        let changed_files = self.pull().await?;
        log::info!("Sync: Repo pulled changes");

        if self.is_file_changed_in_workdir().await? {
            log::info!("Sync: Found changes, adding all...");
            self.add_all().await?;
            log::info!("Sync: Added all files");

            log::info!("Sync: Creating commit...");
            self.commit().await?;
            log::info!("Sync: Created commit");

            log::info!("Sync: Repo pushing changes...");
            self.set_sync_state(SyncState::Pushing);
            self.push().await?;
            log::info!("Sync: Pushed chanes to remote");
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

        utils::do_async(move || {
            let repo = repo.lock().unwrap();

            repo.pull(
                DEFAULT_REMOTE_NAME,
                DEFAULT_AUTHOR_NAME,
                DEFAULT_AUTHOR_EMAIL,
            )
        })
        .await
    }

    async fn is_file_changed_in_workdir(&self) -> anyhow::Result<bool> {
        let repo = self.repository();

        utils::do_async(move || {
            let repo = repo.lock().unwrap();

            repo.is_file_changed_in_workdir()
        })
        .await
    }

    async fn add_all(&self) -> anyhow::Result<()> {
        let repo = self.repository();

        utils::do_async(move || {
            let repo = repo.lock().unwrap();

            repo.add(&["."])
        })
        .await
    }

    async fn commit(&self) -> anyhow::Result<()> {
        let repo = self.repository();

        utils::do_async(move || {
            let repo = repo.lock().unwrap();

            repo.commit("Sync commit", DEFAULT_AUTHOR_NAME, DEFAULT_AUTHOR_EMAIL)
        })
        .await
    }

    async fn push(&self) -> anyhow::Result<()> {
        let repo = self.repository();

        utils::do_async(move || {
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

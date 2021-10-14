use gtk::{
    gio,
    glib::{self, GEnum},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::{sync::Lazy, unsync::OnceCell};
use regex::Regex;

use std::{cell::Cell, path::PathBuf};

use super::{repository::DEFAULT_REMOTE_NAME, Repository};

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
        pub repository: OnceCell<Repository>,
        pub sync_state: Cell<SyncState>,
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
                        "repository",
                        "Repository",
                        "Repository handler",
                        Repository::static_type(),
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
                "repository" => {
                    let repository = value.get().unwrap();
                    self.repository.set(repository).unwrap();
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
                "repository" => self.repository.get().to_value(),
                "sync-state" => obj.sync_state().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct NoteRepository(ObjectSubclass<imp::NoteRepository>);
}

// TODO do not allocate too much strings
impl NoteRepository {
    pub async fn clone(remote_url: String, base_path: &gio::File) -> anyhow::Result<Self> {
        let repository = Repository::clone(remote_url, base_path).await?;
        Ok(glib::Object::new::<Self>(&[("repository", &repository)])
            .expect("Failed to create NoteRepository."))
    }

    pub async fn open(base_path: &gio::File) -> anyhow::Result<Self> {
        let repository = Repository::open(base_path).await?;
        Ok(glib::Object::new::<Self>(&[("repository", &repository)])
            .expect("Failed to create NoteRepository."))
    }

    pub fn validate_remote_url(remote_url: &str) -> bool {
        if remote_url.is_empty() {
            return false;
        }

        RE_VALIDATE_URL.is_match(remote_url)
    }

    // TODO Better way to handle trying to sync multiple times (maybe refactor to use a thread pool)
    // TODO handle conflicts gracefully
    /// Returns the files that changed after the merge from origin
    pub async fn sync(&self) -> anyhow::Result<Vec<(PathBuf, git2::Delta)>> {
        let repo = self.repository();

        if self.sync_state() == SyncState::Pulling {
            log::info!("Currently pulling. Returning...");
            return Ok(Vec::new());
        }

        log::info!("Sync: Repo pulling changes...");
        self.set_sync_state(SyncState::Pulling);
        let changed_files = repo
            .pull(
                DEFAULT_REMOTE_NAME.into(),
                DEFAULT_AUTHOR_NAME.into(),
                DEFAULT_AUTHOR_EMAIL.into(),
            )
            .await?;
        log::info!("Sync: Repo pulled changes");

        if repo.is_file_changed_in_workdir().await? {
            log::info!("Sync: Found changes, adding all...");
            repo.add(vec![".".into()]).await?;
            log::info!("Sync: Creating commit...");
            repo.commit(
                "Sync commit".into(),
                DEFAULT_AUTHOR_NAME.into(),
                DEFAULT_AUTHOR_EMAIL.into(),
            )
            .await?;

            log::info!("Sync: Repo pushing changes...");
            self.set_sync_state(SyncState::Pushing);
            repo.push(DEFAULT_REMOTE_NAME.into()).await?;
            log::info!("Sync: pushed commit");
        } else {
            log::info!("Sync: There is no changed files in directory");
            log::info!("Sync: Skipping pushing and commit...");
        }

        self.set_sync_state(SyncState::Idle);
        Ok(changed_files)
    }

    pub fn connect_remote_changed<F: Fn(&Repository) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.repository().connect_remote_changed(f)
    }

    fn repository(&self) -> Repository {
        let imp = imp::NoteRepository::from_instance(self);
        Clone::clone(imp.repository.get().unwrap())
    }

    fn sync_state(&self) -> SyncState {
        let imp = imp::NoteRepository::from_instance(self);
        imp.sync_state.get()
    }

    fn set_sync_state(&self, sync_state: SyncState) {
        let imp = imp::NoteRepository::from_instance(self);
        imp.sync_state.set(sync_state);
        self.notify("sync-state");
    }
}

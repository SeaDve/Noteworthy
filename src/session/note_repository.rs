use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use once_cell::{sync::Lazy, unsync::OnceCell};

use std::path::PathBuf;

use crate::repository::Repository;

const DEFAULT_REMOTE_NAME: &str = "origin";
const DEFAULT_AUTHOR_NAME: &str = "NoteworthyApp";
const DEFAULT_AUTHOR_EMAIL: &str = "app@noteworthy.io";

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct NoteRepository {
        pub repository: OnceCell<Repository>,
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
                vec![glib::ParamSpec::new_object(
                    "repository",
                    "Repository",
                    "Repository handler",
                    Repository::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "repository" => {
                    let repository = value.get().unwrap();
                    self.repository.set(repository).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "repository" => self.repository.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
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

    /// Returns the files that changed after the merge from origin
    pub async fn sync(&self) -> anyhow::Result<Vec<PathBuf>> {
        let repo = self.repository();

        log::info!("Sync: Repo pulling changes...");
        let changed_files = repo
            .pull(
                DEFAULT_REMOTE_NAME.into(),
                DEFAULT_AUTHOR_NAME.into(),
                DEFAULT_AUTHOR_EMAIL.into(),
            )
            .await?;
        log::info!("Sync: Repo pulled changes");

        if !repo.is_file_changed_in_workdir().await? {
            log::info!("Sync: There is no changed files in directory");
            log::info!("Sync: Skipping pushing and commit...");
            return Ok(changed_files);
        }

        log::info!("Sync: Found changes, creating commit...");
        repo.add(vec![".".into()]).await?;
        repo.commit(
            "Sync commit".into(),
            DEFAULT_AUTHOR_NAME.into(),
            DEFAULT_AUTHOR_EMAIL.into(),
        )
        .await?;
        repo.push(DEFAULT_REMOTE_NAME.into()).await?;
        log::info!("Sync: pushed commit");

        Ok(changed_files)
    }

    fn repository(&self) -> Repository {
        let imp = imp::NoteRepository::from_instance(self);
        Clone::clone(imp.repository.get().unwrap())
    }
}

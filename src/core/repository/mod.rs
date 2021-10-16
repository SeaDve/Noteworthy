mod git2_repo;
pub mod wrapper;
// TODO somehow make wrapper a private mod

use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use once_cell::{sync::Lazy, unsync::OnceCell};

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use self::git2_repo::Git2Repo;

pub static RUNTIME: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Repository {
        pub base_path: OnceCell<gio::File>,
        pub git2_repo: OnceCell<Git2Repo>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Repository {
        const NAME: &'static str = "NwtyRepository";
        type Type = super::Repository;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Repository {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "base-path",
                    "Base Path",
                    "Where the repository is stored locally",
                    gio::File::static_type(),
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
                "base-path" => {
                    let base_path = value.get().unwrap();
                    self.base_path.set(base_path).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "base-path" => self.base_path.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Repository(ObjectSubclass<imp::Repository>);
}

impl Repository {
    pub async fn clone(remote_url: String, directory: &gio::File) -> anyhow::Result<Self> {
        let obj = glib::Object::new::<Self>(&[("base-path", directory)])
            .expect("Failed to create Repository.");

        let path = directory.path().unwrap();
        let repo = Self::run_async(move || wrapper::clone(&path, &remote_url)).await?;
        let imp = imp::Repository::from_instance(&obj);
        imp.git2_repo.set(Git2Repo::new(repo)).unwrap();

        Ok(obj)
    }

    pub async fn open(directory: &gio::File) -> anyhow::Result<Self> {
        let obj = glib::Object::new::<Self>(&[("base-path", directory)])
            .expect("Failed to create Repository.");

        let path = directory.path().unwrap();
        let repo = Self::run_async(move || wrapper::open(&path)).await?;
        let imp = imp::Repository::from_instance(&obj);
        imp.git2_repo.set(Git2Repo::new(repo)).unwrap();

        Ok(obj)
    }

    pub async fn is_file_changed_in_workdir(&self) -> anyhow::Result<bool> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::is_file_changed_in_workdir(&repo)
        })
        .await
    }

    pub async fn is_same(&self, spec_a: String, spec_b: String) -> anyhow::Result<bool> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::is_same(&repo, &spec_a, &spec_b)
        })
        .await
    }

    pub async fn push(&self, remote_name: String) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::push(&repo, &remote_name)
        })
        .await
    }

    pub async fn pull(
        &self,
        remote_name: String,
        author_name: String,
        author_email: String,
    ) -> anyhow::Result<Vec<(PathBuf, git2::Delta)>> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::pull(&repo, &remote_name, &author_name, &author_email)
        })
        .await
    }

    pub async fn commit(
        &self,
        message: String,
        author_name: String,
        author_email: String,
    ) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::commit(&repo, &message, &author_name, &author_email)
        })
        .await
    }

    pub async fn fetch(&self, remote_name: String) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::fetch(&repo, &remote_name)
        })
        .await
    }

    pub async fn add(&self, paths: Vec<PathBuf>) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::add(&repo, &paths)
        })
        .await
    }

    pub async fn remove(&self, paths: Vec<PathBuf>) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::remove(&repo, &paths)
        })
        .await
    }

    pub async fn merge(
        &self,
        source_branch: String,
        author_name: String,
        author_email: String,
    ) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::merge(&repo, &source_branch, None, &author_name, &author_email)
        })
        .await
    }

    async fn run_async<T, F>(f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        RUNTIME.spawn_blocking(f).await.unwrap()
    }

    pub fn base_path(&self) -> gio::File {
        self.property("base-path").unwrap().get().unwrap()
    }

    fn git2_repo(&self) -> Arc<Mutex<git2::Repository>> {
        let imp = imp::Repository::from_instance(self);
        imp.git2_repo.get().unwrap().inner()
    }
}

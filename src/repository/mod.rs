mod git2_repo;
mod wrapper;

use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use once_cell::{sync::Lazy, unsync::OnceCell};
use regex::Regex;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use self::git2_repo::Git2Repo;

static DEFAULT_AUTHOR_NAME: Lazy<String> = Lazy::new(|| String::from("NoteworthyApp"));
static DEFAULT_AUTHOR_EMAIL: Lazy<String> = Lazy::new(|| String::from("app@noteworthy.io"));

static RE_VALIDATE_URL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(git@[\w\.]+)(:(//)?)([\w\.@:/\-~]+)(\.git)(/)?").unwrap());

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

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
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

    pub fn validate_remote_url(remote_url: &str) -> bool {
        if remote_url.is_empty() {
            return false;
        }

        RE_VALIDATE_URL.is_match(remote_url)
    }

    pub async fn push(&self, remote_name: String) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || wrapper::push(&git2_repo, &remote_name)).await?;

        Ok(())
    }

    pub async fn commit(&self, message: String) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            wrapper::commit(
                &git2_repo,
                &message,
                &DEFAULT_AUTHOR_NAME,
                &DEFAULT_AUTHOR_EMAIL,
            )
        })
        .await?;

        Ok(())
    }

    pub async fn fetch(&self, remote_name: String) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || wrapper::fetch(&git2_repo, &remote_name)).await?;

        Ok(())
    }

    pub async fn add(&self, paths: Vec<PathBuf>) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || wrapper::add(&git2_repo, &paths)).await?;

        Ok(())
    }

    pub async fn remove(&self, paths: Vec<PathBuf>) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || wrapper::remove(&git2_repo, &paths)).await?;

        Ok(())
    }

    pub async fn merge(&self, source_branch: String) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            wrapper::merge(
                &git2_repo,
                &source_branch,
                &DEFAULT_AUTHOR_NAME,
                &DEFAULT_AUTHOR_EMAIL,
            )
        })
        .await?;

        Ok(())
    }

    pub async fn pull(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn run_async<F, T>(f: F) -> anyhow::Result<T>
    where
        F: FnOnce() -> anyhow::Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let (sender, receiver) = futures::channel::oneshot::channel();

        thread::spawn(move || {
            let res = f();
            if sender.send(res).is_err() {
                // why git2::Repository doesn't Debug??
                panic!("Failed to send");
            }
        });

        let res = receiver.await.unwrap()?;

        Ok(res)
    }

    fn git2_repo(&self) -> Arc<Mutex<git2::Repository>> {
        let imp = imp::Repository::from_instance(self);
        imp.git2_repo.get().unwrap().inner()
    }
}

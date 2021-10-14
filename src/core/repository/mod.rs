mod git2_repo;
mod wrapper;

use gtk::{
    gio,
    glib::{self, clone, subclass::Signal},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::{sync::Lazy, unsync::OnceCell};

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use self::git2_repo::Git2Repo;

pub const DEFAULT_REMOTE_NAME: &str = "origin";
const DEFAULT_SLEEP_TIME_SECS: u64 = 5;

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
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("remote-changed", &[], <()>::static_type().into()).build()]
            });
            SIGNALS.as_ref()
        }

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

        obj.setup_daemon();
        Ok(obj)
    }

    pub async fn open(directory: &gio::File) -> anyhow::Result<Self> {
        let obj = glib::Object::new::<Self>(&[("base-path", directory)])
            .expect("Failed to create Repository.");

        let path = directory.path().unwrap();
        let repo = Self::run_async(move || wrapper::open(&path)).await?;
        let imp = imp::Repository::from_instance(&obj);
        imp.git2_repo.set(Git2Repo::new(repo)).unwrap();

        obj.setup_daemon();
        Ok(obj)
    }

    pub async fn is_file_changed_in_workdir(&self) -> anyhow::Result<bool> {
        let git2_repo = self.git2_repo();

        let res = Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::is_file_changed_in_workdir(&repo)
        })
        .await?;

        Ok(res)
    }

    pub async fn is_same(&self, spec_a: String, spec_b: String) -> anyhow::Result<bool> {
        let git2_repo = self.git2_repo();

        let res = Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::is_same(&repo, &spec_a, &spec_b)
        })
        .await?;

        Ok(res)
    }

    pub async fn push(&self, remote_name: String) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::push(&repo, &remote_name)
        })
        .await?;

        Ok(())
    }

    pub async fn pull(
        &self,
        remote_name: String,
        author_name: String,
        author_email: String,
    ) -> anyhow::Result<Vec<(PathBuf, git2::Delta)>> {
        let git2_repo = self.git2_repo();

        let changed_files = Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::pull(&repo, &remote_name, &author_name, &author_email)
        })
        .await?;

        Ok(changed_files)
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
        .await?;

        Ok(())
    }

    pub async fn fetch(&self, remote_name: String) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::fetch(&repo, &remote_name)
        })
        .await?;

        Ok(())
    }

    pub async fn add(&self, paths: Vec<PathBuf>) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::add(&repo, &paths)
        })
        .await?;

        Ok(())
    }

    pub async fn remove(&self, paths: Vec<PathBuf>) -> anyhow::Result<()> {
        let git2_repo = self.git2_repo();

        Self::run_async(move || {
            let repo = git2_repo.lock().unwrap();
            wrapper::remove(&repo, &paths)
        })
        .await?;

        Ok(())
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
        .await?;

        Ok(())
    }

    pub fn connect_remote_changed<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_local("remote-changed", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);
            None
        })
        .unwrap()
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

    fn setup_daemon(&self) {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT_IDLE);

        let base_path = self.base_path().path().unwrap();
        thread::spawn(move || match wrapper::open(&base_path) {
            Ok(repo) => {
                log::info!("Starting Daemon...");

                loop {
                    wrapper::fetch(&repo, DEFAULT_REMOTE_NAME).unwrap_or_else(|err| {
                        log::error!("Daemon: Failed to fetch to origin: {}", err)
                    });
                    if let Ok(is_same) = wrapper::is_same(&repo, "HEAD", "FETCH_HEAD") {
                        sender.send(is_same).unwrap_or_else(|err| {
                            log::error!("Daemon: Failed to send message to channel: {}", err)
                        });
                    } else {
                        log::error!("Daemon: Failed to compare HEAD from FETCH_HEAD")
                    }
                    thread::sleep(Duration::from_secs(DEFAULT_SLEEP_TIME_SECS));
                }
            }
            Err(err) => {
                log::error!(
                    "Daemon: Failed to open repo with path {}: {}",
                    base_path.display(),
                    err
                );
            }
        });

        receiver.attach(
            None,
            clone!(@weak self as obj => @default-return glib::Continue(true), move |is_same| {
                if !is_same {
                    obj.emit_by_name("remote-changed", &[]).unwrap();
                }
                glib::Continue(true)
            }),
        );
    }

    fn git2_repo(&self) -> Arc<Mutex<git2::Repository>> {
        let imp = imp::Repository::from_instance(self);
        imp.git2_repo.get().unwrap().inner()
    }

    fn base_path(&self) -> gio::File {
        self.property("base-path").unwrap().get().unwrap()
    }
}

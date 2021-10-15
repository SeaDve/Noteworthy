use gtk::{
    gio,
    glib::{self, clone, subclass::Signal},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::{sync::Lazy, unsync::OnceCell};

use std::time::Duration;

use super::repository::wrapper;
use crate::RUNTIME;

const DEFAULT_SLEEP_TIME_SECS: u64 = 5;

mod imp {
    use super::*;

    #[derive(Default, Debug)]
    pub struct RepositoryWatcher {
        pub base_path: OnceCell<gio::File>,
        pub remote_name: OnceCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RepositoryWatcher {
        const NAME: &'static str = "NwtyRepositoryWatcher";
        type Type = super::RepositoryWatcher;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for RepositoryWatcher {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("remote-changed", &[], <()>::static_type().into()).build()]
            });
            SIGNALS.as_ref()
        }

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
                    glib::ParamSpec::new_string(
                        "remote-name",
                        "Remote Name",
                        "Remote name where the repo will be stored (e.g. origin)",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
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
                "remote-name" => {
                    let remote_name = value.get().unwrap();
                    self.remote_name.set(remote_name).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "base-path" => self.base_path.get().to_value(),
                "remote-name" => self.remote_name.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup();
        }
    }
}

glib::wrapper! {
    pub struct RepositoryWatcher(ObjectSubclass<imp::RepositoryWatcher>);
}

impl RepositoryWatcher {
    pub fn new(base_path: &gio::File, remote_name: &str) -> Self {
        glib::Object::new::<Self>(&[("base-path", &base_path), ("remote-name", &remote_name)])
            .expect("Failed to create RepositoryWatcher.")
    }

    pub fn connect_remote_changed<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_local("remote-changed", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);
            None
        })
        .unwrap()
    }

    fn base_path(&self) -> gio::File {
        self.property("base-path").unwrap().get().unwrap()
    }

    fn remote_name(&self) -> String {
        self.property("remote-name").unwrap().get().unwrap()
    }

    fn setup(&self) {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT_IDLE);

        let base_path = self.base_path().path().unwrap();
        let remote_name = self.remote_name();

        RUNTIME.spawn(async move {
            match wrapper::open(&base_path) {
                Ok(repo) => {
                    log::info!("Starting watcher thread...");

                    loop {
                        wrapper::fetch(&repo, &remote_name).unwrap_or_else(|err| {
                            log::error!("Failed to fetch to origin: {}", err);
                        });
                        if let Ok(is_same) = wrapper::is_same(&repo, "HEAD", "FETCH_HEAD") {
                            sender.send(is_same).unwrap_or_else(|err| {
                                log::error!("Failed to send message to channel: {}", err);
                            });
                        } else {
                            log::error!("Failed to compare HEAD from FETCH_HEAD");
                        }
                        tokio::time::sleep(Duration::from_secs(DEFAULT_SLEEP_TIME_SECS)).await;
                    }
                }
                Err(err) => {
                    log::error!(
                        "Failed to open repo with path {}: {}",
                        base_path.display(),
                        err
                    );
                }
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
}

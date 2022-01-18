use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use once_cell::unsync::OnceCell;

use std::{thread, time::Duration};

use super::Repository;

const DEFAULT_SLEEP_TIME_SECS: u64 = 3;

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use once_cell::sync::Lazy;

    #[derive(Default, Debug)]
    pub struct RepositoryWatcher {
        pub base_path: OnceCell<gio::File>,
        pub remote_name: OnceCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RepositoryWatcher {
        const NAME: &'static str = "NwtyRepositoryWatcher";
        type Type = super::RepositoryWatcher;
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
                    glib::ParamSpecObject::new(
                        "base-path",
                        "Base Path",
                        "Where the repository is stored locally",
                        gio::File::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecString::new(
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
        glib::Object::new(&[("base-path", &base_path), ("remote-name", &remote_name)])
            .expect("Failed to create RepositoryWatcher.")
    }

    pub fn connect_remote_changed<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_local("remote-changed", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);
            None
        })
    }

    fn base_path(&self) -> gio::File {
        self.property("base-path")
    }

    fn remote_name(&self) -> String {
        self.property("remote-name")
    }

    fn setup(&self) {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT_IDLE);

        let base_path = self.base_path().path().unwrap();
        let remote_name = self.remote_name();

        // FIXME join and end the thread properly when `self` is dropped
        thread::spawn(move || match Repository::open(&base_path) {
            Ok(repo) => {
                log::info!("Starting watcher thread...");

                loop {
                    repo.fetch(&remote_name).unwrap_or_else(|err| {
                        log::error!("Failed to fetch to origin: {:?}", err);
                    });
                    if let Ok(is_same) = repo.is_same("HEAD", "FETCH_HEAD") {
                        sender.send(is_same).unwrap_or_else(|err| {
                            log::error!("Failed to send message to channel: {:?}", err);
                        });
                    } else {
                        log::error!("Failed to compare HEAD from FETCH_HEAD");
                    }
                    thread::sleep(Duration::from_secs(DEFAULT_SLEEP_TIME_SECS));
                }
            }
            Err(err) => {
                log::error!(
                    "Failed to open repo with path `{}`: {:?}",
                    base_path.display(),
                    err
                );
            }
        });

        receiver.attach(
            None,
            clone!(@weak self as obj => @default-return Continue(false), move |is_same| {
                if !is_same {
                    obj.emit_by_name::<()>("remote-changed", &[]);
                }
                Continue(true)
            }),
        );
    }
}

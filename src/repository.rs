use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use once_cell::unsync::OnceCell;

use std::thread;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Repository {
        pub remote_url: OnceCell<String>,
        pub local_path: OnceCell<gio::File>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Repository {
        const NAME: &'static str = "NwtyRepository";
        type Type = super::Repository;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Repository {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_string(
                        "remote-url",
                        "Remote Url",
                        "Remote URL of the repository",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "local-path",
                        "Local Path",
                        "Where the repository is stored locally",
                        gio::File::static_type(),
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
                "remote-url" => {
                    let remote_url = value.get().unwrap();
                    self.remote_url.set(remote_url).unwrap();
                }
                "local-path" => {
                    let local_path = value.get().unwrap();
                    self.local_path.set(local_path).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "remote-url" => self.remote_url.get().to_value(),
                "local-path" => self.local_path.get().to_value(),
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
    pub fn new(remote_url: &String, local_path: &gio::File) -> Self {
        glib::Object::new::<Self>(&[("remote-url", remote_url), ("local-path", local_path)])
            .expect("Failed to create Repository.")
    }

    pub fn remote_url(&self) -> String {
        self.property("remote-url").unwrap().get().unwrap()
    }

    pub fn local_path(&self) -> gio::File {
        self.property("local-path").unwrap().get().unwrap()
    }

    pub async fn clone(&self, passphrase: Option<&str>) -> anyhow::Result<()> {
        let (sender, receiver) = futures::channel::oneshot::channel();

        // FIXME dont clone
        let passphrase = passphrase.map(std::string::ToString::to_string);
        let remote_url = self.remote_url();
        let local_path = self.local_path();

        thread::spawn(move || {
            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.credentials(|_url, username_from_url, _allowed_types| {
                let mut ssh_key_path = glib::home_dir();
                ssh_key_path.push(".ssh/id_ed25519");

                log::info!("Credential callback");

                git2::Cred::ssh_key(
                    username_from_url.unwrap(),
                    None,
                    &ssh_key_path,
                    passphrase.as_deref(),
                )
            });
            callbacks.transfer_progress(|progress| {
                dbg!(progress.total_objects());
                dbg!(progress.indexed_objects());
                dbg!(progress.received_objects());
                dbg!(progress.local_objects());
                dbg!(progress.total_deltas());
                dbg!(progress.indexed_deltas());
                dbg!(progress.received_bytes());
                true
            });

            log::info!("Preparing to clone");

            let mut fo = git2::FetchOptions::new();
            fo.remote_callbacks(callbacks);

            let mut builder = git2::build::RepoBuilder::new();
            builder.fetch_options(fo);

            let res = builder.clone(&remote_url, &local_path.path().unwrap());

            sender.send(res)
        });

        receiver.await.unwrap()?;

        Ok(())
    }

    pub fn commit(
        &self,
        author_name: &str,
        author_email: &str,
        message: &str,
    ) -> anyhow::Result<()> {
        let repo = git2::Repository::open(self.local_path().path().unwrap())?;

        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let signature = git2::Signature::now(author_name, author_email)?;

        match repo.refname_to_id("HEAD") {
            Ok(parent_id) => {
                let parent_commit = repo.find_commit(parent_id)?;
                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    message,
                    &tree,
                    &[&parent_commit],
                )?;
            }
            Err(err) => {
                repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])?;
                log::warn!("Failed to refname_to_id: {}", err);
            }
        };

        Ok(())
    }
}

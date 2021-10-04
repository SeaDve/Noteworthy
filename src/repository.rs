use gtk::glib;

use std::{path::PathBuf, thread};

pub struct Repository {
    remote_url: String,
    local_path: PathBuf,
}

impl Repository {
    pub fn new(remote_url: String, local_path: PathBuf) -> Self {
        Self {
            remote_url,
            local_path,
        }
    }

    pub async fn clone(&self, passphrase: Option<&str>) -> anyhow::Result<()> {
        let (sender, receiver) = futures::channel::oneshot::channel();

        // FIXME dont clone
        let passphrase = passphrase.map(std::string::ToString::to_string);
        let remote_url = self.remote_url.clone();
        let local_path = self.local_path.clone();

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

            let res = builder.clone(&remote_url, &local_path);

            sender.send(match res {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            })
        });

        Ok(receiver.await.unwrap()?)
    }
}

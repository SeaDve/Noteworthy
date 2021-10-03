use git2::{Cred, RemoteCallbacks, Repository as Git2Repository};
use gtk::glib;
use once_cell::sync::OnceCell;

use std::path::PathBuf;

pub struct Repository {
    remote_url: String,
    local_path: PathBuf,
    inner: OnceCell<Git2Repository>,
}

impl Repository {
    pub fn new(remote_url: String, local_path: PathBuf) -> Self {
        Self {
            remote_url,
            local_path,
            inner: OnceCell::new(),
        }
    }

    pub fn clone(&self, passphrase: Option<&str>) -> anyhow::Result<()> {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            let mut ssh_key_path = glib::home_dir();
            ssh_key_path.push(".ssh/id_ed25519");

            log::info!("Credential callback");

            Cred::ssh_key(username_from_url.unwrap(), None, &ssh_key_path, passphrase)
        });

        log::info!("Preparing to clone");

        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);

        let repo = builder.clone(&self.remote_url, &self.local_path)?;
        if self.inner.set(repo).is_err() {
            panic!("inner repo already set.");
        }

        log::info!("Clone done");

        Ok(())
    }
}

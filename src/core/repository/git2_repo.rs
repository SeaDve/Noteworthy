use tokio::sync::Mutex;

use std::sync::Arc;

// Hack because git2::Repository doesn't do Debug
pub struct Git2Repo(Arc<Mutex<git2::Repository>>);

impl std::fmt::Debug for Git2Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("git2::Repository inside a Git2Repo")
    }
}

impl Git2Repo {
    pub fn new(inner: git2::Repository) -> Self {
        Self(Arc::new(Mutex::new(inner)))
    }

    pub fn inner(&self) -> Arc<Mutex<git2::Repository>> {
        Arc::clone(&self.0)
    }
}

use gtk::glib;

use std::{future::Future, path::PathBuf};

use crate::THREAD_POOL;

pub fn default_notes_dir() -> PathBuf {
    let mut data_dir = glib::user_data_dir();
    data_dir.push("Notes");
    data_dir
}

pub async fn spawn_blocking<T, F>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    THREAD_POOL.push_future(f).unwrap().await
}

pub fn spawn<F>(f: F)
where
    F: Future<Output = ()> + 'static,
{
    spawn_with_priority(glib::PRIORITY_DEFAULT, f);
}

pub fn spawn_with_priority<F>(priority: glib::Priority, f: F)
where
    F: Future<Output = ()> + 'static,
{
    let ctx = glib::MainContext::default();
    ctx.spawn_local_with_priority(priority, f);
}

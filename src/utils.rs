use gtk::glib;

use std::path::PathBuf;

use crate::THREAD_POOL;

pub fn default_notes_dir() -> PathBuf {
    let mut data_dir = glib::user_data_dir();
    data_dir.push("Notes");
    data_dir
}

pub async fn do_async<T, F>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    THREAD_POOL.push_future(f).unwrap().await
}

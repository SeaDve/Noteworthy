use gtk::glib;

use std::path::PathBuf;

// Taken from fractal-next GPLv3
// See https://gitlab.gnome.org/GNOME/fractal/-/blob/fractal-next/src/utils.rs
#[macro_export]
macro_rules! spawn {
    ($future:expr) => {
        let ctx = glib::MainContext::default();
        ctx.spawn_local($future);
    };
    ($priority:expr, $future:expr) => {
        let ctx = glib::MainContext::default();
        ctx.spawn_local_with_priority($priority, $future);
    };
}

#[macro_export]
macro_rules! spawn_blocking {
    ($future:expr) => {
        crate::THREAD_POOL.push_future($future).unwrap()
    };
}

pub fn default_notes_dir() -> PathBuf {
    let mut data_dir = glib::user_data_dir();
    data_dir.push("Notes");
    data_dir
}

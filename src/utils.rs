use gtk::glib;

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

// Taken from fractal-next GPLv3
// See https://gitlab.gnome.org/GNOME/fractal/-/blob/fractal-next/src/utils.rs
/// Spawns a future in the main context
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

/// Pushes a function to be executed in the main thread pool
#[macro_export]
macro_rules! spawn_blocking {
    ($function:expr) => {
        $crate::THREAD_POOL.push_future($function).unwrap()
    };
}

pub fn default_notes_dir() -> PathBuf {
    let mut data_dir = glib::user_data_dir();
    data_dir.push("Notes");
    data_dir
}

pub fn generate_unique_path(
    base_path: impl AsRef<Path>,
    file_name_prefix: &str,
    extension: Option<impl AsRef<OsStr>>,
) -> PathBuf {
    let formatted_time = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S-%f");
    let file_name = format!("{}-{}", file_name_prefix, formatted_time);

    let mut path = base_path.as_ref().join(file_name);

    if let Some(extension) = extension {
        path.set_extension(extension);
    }

    assert!(!path.exists());

    path
}

use gtk::{
    gio::{self, prelude::*},
    glib,
};

use std::path::{Path, PathBuf};

use crate::utils;

#[derive(Debug)]
pub struct AudioRecording {
    file: gio::File,
}

impl AudioRecording {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        let path = utils::generate_unique_path(base_path.as_ref(), "AudioRecording", Some("ogg"));

        Self {
            file: gio::File::for_path(path),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.file.path().unwrap()
    }

    pub async fn delete(&self) -> Result<(), glib::Error> {
        self.file.delete_future(glib::PRIORITY_DEFAULT_IDLE).await
    }

    pub fn into_file(self) -> gio::File {
        self.file
    }
}

use gtk::{gio, prelude::*};

#[derive(Debug, PartialEq)]
pub enum AttachmentKind {
    Ogg,
    Png,
    Other,
}

impl AttachmentKind {
    pub fn for_file(file: &gio::File) -> Self {
        // TODO what if the file has no extension

        let path = file.path().unwrap();
        let extension = path
            .extension()
            .unwrap()
            .to_str()
            .expect("extension not in valid utf-8");

        match extension {
            "ogg" => Self::Ogg,
            "png" => Self::Png,
            _ => Self::Other,
        }
    }
}

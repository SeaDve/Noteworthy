use gtk::{gio, prelude::*};

#[derive(Debug)]
pub enum AttachmentKind {
    Ogg,
    Other,
}

impl AttachmentKind {
    pub fn for_file(file: &gio::File) -> Self {
        let path = file.path().unwrap();
        let extension = path
            .extension()
            .unwrap()
            .to_str()
            .expect("extension not in valid utf-8");

        match extension {
            "ogg" => Self::Ogg,
            _ => Self::Other,
        }
    }
}

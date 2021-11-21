use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttachmentKind {
    Ogg,
    Png,
    Other,
}

impl AttachmentKind {
    pub fn for_path(path: &Path) -> Self {
        // TODO what if the path has no extension
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

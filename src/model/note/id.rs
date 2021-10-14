use std::path::Path;

// TODO optimize this (Reduce size of id in generating unique file name in note/mod.rs)
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Id {
    id: Box<str>,
}

impl Id {
    pub fn from_path(path: &Path) -> Self {
        Self {
            id: path.file_stem().unwrap().to_str().unwrap().into(),
        }
    }
}

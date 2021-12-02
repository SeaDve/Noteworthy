use std::path::Path;

// TODO optimize this (Reduce size of id in generating unique file name in utils.rs)
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Id {
    id: Box<str>,
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.id)
    }
}

impl Id {
    pub fn from_path(path: &Path) -> Self {
        Self {
            id: path.file_stem().unwrap().to_str().unwrap().into(),
        }
    }
}

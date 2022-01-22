use std::{ffi::OsStr, path::Path};

// TODO optimize this (Reduce size of id in generating unique file name in utils.rs)
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct NoteId {
    id: Box<OsStr>,
}

impl std::fmt::Debug for NoteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.id, f)
    }
}

impl NoteId {
    pub fn for_path(path: impl AsRef<Path>) -> Self {
        Self {
            id: Box::from(path.as_ref().file_stem().unwrap()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn hash_map() {
        let mut hash_map = HashMap::new();

        let id_0 = NoteId::for_path("Path0");
        hash_map.insert(&id_0, 0);

        let id_1 = NoteId::for_path("Path1");
        hash_map.insert(&id_1, 1);

        let id_2 = NoteId::for_path("Path2");
        hash_map.insert(&id_2, 2);

        assert_eq!(hash_map.get(&id_0), Some(&0));
        assert_eq!(hash_map.get(&id_1), Some(&1));
        assert_eq!(hash_map.get(&NoteId::for_path("Path2")), Some(&2));
    }
}

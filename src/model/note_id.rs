use std::path::Path;

// TODO optimize this (Reduce size of id in generating unique file name in utils.rs)
#[derive(Hash, PartialEq, Eq)]
pub struct NoteId {
    id: Box<str>,
}

impl std::fmt::Debug for NoteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

impl NoteId {
    pub fn from_path(path: &Path) -> Self {
        Self {
            id: path.file_stem().unwrap().to_str().unwrap().into(),
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

        let id_0 = NoteId::from_path(&Path::new("Path0"));
        hash_map.insert(&id_0, 0);

        let id_1 = NoteId::from_path(&Path::new("Path1"));
        hash_map.insert(&id_1, 1);

        let id_2 = NoteId::from_path(&Path::new("Path2"));
        hash_map.insert(&id_2, 2);

        assert_eq!(hash_map.get(&id_0), Some(&0));
        assert_eq!(hash_map.get(&id_1), Some(&1));
        assert_eq!(
            hash_map.get(&NoteId::from_path(&Path::new("Path2"))),
            Some(&2)
        );
    }
}

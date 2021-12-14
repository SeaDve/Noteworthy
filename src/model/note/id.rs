use std::{ffi::OsStr, path::Path};

// TODO optimize this (Reduce size of id in generating unique file name in utils.rs)
#[derive(Hash, PartialEq, Eq)]
pub struct Id {
    id: Box<OsStr>,
}

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

impl Id {
    pub fn from_path(path: impl AsRef<Path>) -> Self {
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

        let id_0 = Id::from_path("Path0");
        hash_map.insert(&id_0, 0);

        let id_1 = Id::from_path("Path1");
        hash_map.insert(&id_1, 1);

        let id_2 = Id::from_path("Path2");
        hash_map.insert(&id_2, 2);

        assert_eq!(hash_map.get(&id_0), Some(&0));
        assert_eq!(hash_map.get(&id_1), Some(&1));
        assert_eq!(hash_map.get(&Id::from_path("Path2")), Some(&2));
    }
}

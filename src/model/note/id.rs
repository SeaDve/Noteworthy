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

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn hash_map() {
        let mut hash_map = HashMap::new();

        let id_0 = Id::from_path(&Path::new("Path0"));
        hash_map.insert(&id_0, 0);

        let id_1 = Id::from_path(&Path::new("Path1"));
        hash_map.insert(&id_1, 1);

        let id_2 = Id::from_path(&Path::new("Path2"));
        hash_map.insert(&id_2, 2);

        assert_eq!(hash_map.get(&id_0), Some(&0));
        assert_eq!(hash_map.get(&id_1), Some(&1));
        assert_eq!(hash_map.get(&Id::from_path(&Path::new("Path2"))), Some(&2));
    }
}

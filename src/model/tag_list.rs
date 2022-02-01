use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::cell::RefCell;

use super::Tag;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct TagList {
        pub list: RefCell<IndexMap<String, Tag>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TagList {
        const NAME: &'static str = "NwtyTagList";
        type Type = super::TagList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for TagList {}

    impl ListModelImpl for TagList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            Tag::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .get_index(position as usize)
                .map(|(_, v)| v.upcast_ref::<glib::Object>())
                .cloned()
        }
    }
}

glib::wrapper! {
    pub struct TagList(ObjectSubclass<imp::TagList>)
        @implements gio::ListModel;
}

impl TagList {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create TagList.")
    }

    pub fn append(&self, tag: Tag) -> anyhow::Result<()> {
        let tag_name = tag.name();

        anyhow::ensure!(!tag_name.is_empty(), "Tag name cannot be empty");

        // FIXME disconnect this when !is_name_appended
        // audio_player_handler, attachment_list, note_list, note_tag_list, and tag_list also
        // have this problem
        tag.connect_name_notify(clone!(@weak self as obj => move |tag| {
            if let Some(position) = obj.get_index_of(tag) {
                obj.items_changed(position as u32, 1, 1);
            }
        }));

        let is_name_appended = self.imp().list.borrow_mut().insert(tag_name, tag).is_none();

        anyhow::ensure!(is_name_appended, "Cannot append existing tag name");

        self.items_changed(self.n_items() - 1, 0, 1);

        Ok(())
    }

    pub fn remove(&self, tag: &Tag) -> anyhow::Result<()> {
        let tag_name = tag.name();

        let removed = self.imp().list.borrow_mut().shift_remove_full(&tag_name);

        if let Some((position, _, _)) = removed {
            self.items_changed(position as u32, 1, 0);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Cannot remove tag that does not exist"))
        }
    }

    pub fn rename_tag(&self, tag: &Tag, new_name: &str) -> anyhow::Result<()> {
        anyhow::ensure!(
            !self.contains_with_name(new_name),
            "Cannot rename a tag to a name that already exist"
        );
        anyhow::ensure!(!new_name.is_empty(), "Tag name cannot be empty");

        let previous_name = tag.name();

        {
            let mut list = self.imp().list.borrow_mut();
            // Put new name at the end
            assert!(list.insert(new_name.to_string(), tag.clone()).is_none());
            // Remove the old name at the list and replace it with name from the end
            assert!(list.swap_remove(&previous_name).is_some());
            // Might replace this in the future with https://github.com/rust-lang/rust/issues/44286
        }

        tag.set_name(new_name);

        Ok(())
    }

    pub fn contains(&self, tag: &Tag) -> bool {
        self.contains_with_name(&tag.name())
    }

    pub fn contains_with_name(&self, name: &str) -> bool {
        self.imp().list.borrow().contains_key(name)
    }

    pub fn get_with_name(&self, name: &str) -> Option<Tag> {
        self.imp().list.borrow().get(name).cloned()
    }

    pub fn is_valid_name(&self, name: &str) -> bool {
        !self.contains_with_name(name) && !name.is_empty()
    }

    fn get_index_of(&self, tag: &Tag) -> Option<usize> {
        self.imp().list.borrow().get_index_of(&tag.name())
    }
}

impl std::iter::FromIterator<Tag> for TagList {
    fn from_iter<I: IntoIterator<Item = Tag>>(iter: I) -> Self {
        let tag_list = Self::new();

        for tag in iter {
            if let Err(err) = tag_list.append(tag) {
                log::warn!("Error appending a tag, skipping: {:?}", err);
            }
        }

        tag_list
    }
}

impl Serialize for TagList {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.imp().list.borrow().keys())
    }
}

impl<'de> Deserialize<'de> for TagList {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let tags: Vec<Tag> = Vec::deserialize(deserializer)?;

        let tag_list = tags.into_iter().collect::<Self>();

        Ok(tag_list)
    }
}

impl Default for TagList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn append() {
        let tag_list = TagList::new();
        assert!(!tag_list.contains_with_name("A"));

        assert!(tag_list.append(Tag::new("A")).is_ok());
        assert_eq!(tag_list.n_items(), 1);
        assert!(tag_list.contains_with_name("A"));
    }

    #[test]
    fn multiple_append() {
        let tag_list = TagList::new();
        assert!(!tag_list.contains_with_name("A"));
        assert!(!tag_list.contains_with_name("a"));
        assert!(!tag_list.contains_with_name("B"));
        assert!(!tag_list.contains_with_name("b"));

        assert!(tag_list.append(Tag::new("A")).is_ok());
        assert_eq!(tag_list.n_items(), 1);
        assert!(tag_list.contains_with_name("A"));

        assert!(tag_list.append(Tag::new("a")).is_ok());
        assert_eq!(tag_list.n_items(), 2);
        assert!(tag_list.contains_with_name("a"));

        assert!(tag_list.append(Tag::new("B")).is_ok());
        assert_eq!(tag_list.n_items(), 3);
        assert!(tag_list.contains_with_name("B"));

        assert!(tag_list.append(Tag::new("b")).is_ok());
        assert_eq!(tag_list.n_items(), 4);
        assert!(tag_list.contains_with_name("b"));
    }

    #[test]
    fn double_same_append() {
        let tag_list = TagList::new();
        assert!(tag_list.append(Tag::new("A")).is_ok());
        assert!(tag_list.append(Tag::new("A")).is_err());
        assert_eq!(tag_list.n_items(), 1);
    }

    #[test]
    fn remove() {
        let tag_list = TagList::new();
        let tag = Tag::new("A");
        assert!(!tag_list.contains_with_name("A"));

        assert!(tag_list.append(tag.clone()).is_ok());
        assert!(tag_list.contains_with_name("A"));
        assert_eq!(tag_list.n_items(), 1);

        assert!(tag_list.remove(&tag).is_ok());
        assert!(!tag_list.contains_with_name("A"));
        assert_eq!(tag_list.n_items(), 0);
    }

    #[test]
    fn multiple_remove() {
        let tag_list = TagList::new();
        let tag_a = Tag::new("A");
        let tag_a2 = Tag::new("a");
        let tag_b = Tag::new("B");
        let tag_b2 = Tag::new("b");
        assert!(!tag_list.contains_with_name("A"));
        assert!(!tag_list.contains_with_name("a"));
        assert!(!tag_list.contains_with_name("B"));
        assert!(!tag_list.contains_with_name("b"));

        assert!(tag_list.append(tag_a.clone()).is_ok());
        assert!(tag_list.contains_with_name("A"));
        assert!(tag_list.append(tag_a2.clone()).is_ok());
        assert!(tag_list.contains_with_name("a"));
        assert!(tag_list.append(tag_b.clone()).is_ok());
        assert!(tag_list.contains_with_name("B"));
        assert!(tag_list.append(tag_b2.clone()).is_ok());
        assert!(tag_list.contains_with_name("b"));
        assert_eq!(tag_list.n_items(), 4);

        assert!(tag_list.remove(&tag_a).is_ok());
        assert_eq!(tag_list.n_items(), 3);
        assert!(!tag_list.contains_with_name("A"));
        assert!(tag_list.remove(&tag_a2).is_ok());
        assert_eq!(tag_list.n_items(), 2);
        assert!(!tag_list.contains_with_name("a"));
        assert!(tag_list.remove(&tag_b).is_ok());
        assert_eq!(tag_list.n_items(), 1);
        assert!(!tag_list.contains_with_name("B"));
        assert!(tag_list.remove(&tag_b2).is_ok());
        assert_eq!(tag_list.n_items(), 0);
        assert!(!tag_list.contains_with_name("b"));
    }

    #[test]
    fn double_same_remove() {
        let tag_list = TagList::new();
        let tag = Tag::new("A");

        assert!(tag_list.append(tag.clone()).is_ok());
        assert_eq!(tag_list.n_items(), 1);

        assert!(tag_list.remove(&tag).is_ok());
        assert_eq!(tag_list.n_items(), 0);

        assert!(tag_list.remove(&tag).is_err());
        assert_eq!(tag_list.n_items(), 0);
    }

    #[test]
    fn rename() {
        let tag_list = TagList::new();
        let tag = Tag::new("A");
        assert!(!tag_list.contains_with_name("A"));

        assert!(tag_list.append(tag.clone()).is_ok());
        assert!(tag_list.contains_with_name("A"));
        assert!(tag_list.contains(&tag));
        assert_eq!(tag.name(), "A");
        assert_eq!(tag_list.n_items(), 1);

        assert!(tag_list.rename_tag(&tag, "Ab").is_ok());
        assert!(!tag_list.contains_with_name("A"));
        assert!(tag_list.contains_with_name("Ab"));
        assert!(tag_list.contains(&tag));
        assert_eq!(tag.name(), "Ab");
        assert_eq!(tag_list.n_items(), 1);

        assert!(tag_list.rename_tag(&tag, "A").is_ok());
        assert!(!tag_list.contains_with_name("Ab"));
        assert!(tag_list.contains_with_name("A"));
        assert!(tag_list.contains(&tag));
        assert_eq!(tag.name(), "A");
        assert_eq!(tag_list.n_items(), 1);
    }

    #[test]
    fn rename_tag_empty() {
        let tag_list = TagList::new();
        let tag = Tag::new("A");
        assert!(tag_list.append(tag.clone()).is_ok());
        assert_eq!(tag.name(), "A");

        assert!(tag_list.rename_tag(&tag, "AA").is_ok());
        assert_eq!(tag.name(), "AA");

        assert!(tag_list.rename_tag(&tag, "").is_err());
        assert_eq!(tag.name(), "AA");

        assert_eq!(tag_list.n_items(), 1);
    }

    #[test]
    fn rename_multiple_items() {
        let tag_list = TagList::new();

        let tag_a = Tag::new("A");
        assert!(tag_list.append(tag_a.clone()).is_ok());
        assert_eq!(tag_a.name(), "A");
        let tag_a_index = tag_list.get_index_of(&tag_a).unwrap();

        let tag_b = Tag::new("B");
        assert!(tag_list.append(tag_b.clone()).is_ok());
        assert_eq!(tag_b.name(), "B");
        let tag_b_index = tag_list.get_index_of(&tag_b).unwrap();

        let tag_c = Tag::new("C");
        assert!(tag_list.append(tag_c.clone()).is_ok());
        assert_eq!(tag_c.name(), "C");
        let tag_c_index = tag_list.get_index_of(&tag_c).unwrap();

        let tag_d = Tag::new("D");
        assert!(tag_list.append(tag_d.clone()).is_ok());
        assert_eq!(tag_d.name(), "D");
        let tag_d_index = tag_list.get_index_of(&tag_d).unwrap();

        assert_eq!(tag_list.n_items(), 4);

        assert!(tag_list.contains_with_name("A"));
        assert!(tag_list.rename_tag(&tag_a, "AA").is_ok());
        assert!(tag_list.contains_with_name("AA"));
        assert!(!tag_list.contains_with_name("A"));
        assert_eq!(tag_a.name(), "AA");

        assert!(tag_list.contains_with_name("B"));
        assert!(tag_list.rename_tag(&tag_b, "BB").is_ok());
        assert!(tag_list.contains_with_name("BB"));
        assert!(!tag_list.contains_with_name("B"));
        assert_eq!(tag_b.name(), "BB");

        assert!(tag_list.contains_with_name("C"));
        assert!(tag_list.rename_tag(&tag_c, "CC").is_ok());
        assert!(tag_list.contains_with_name("CC"));
        assert!(!tag_list.contains_with_name("C"));
        assert_eq!(tag_c.name(), "CC");

        assert!(tag_list.contains_with_name("D"));
        assert!(tag_list.rename_tag(&tag_d, "DD").is_ok());
        assert!(tag_list.contains_with_name("DD"));
        assert!(!tag_list.contains_with_name("D"));
        assert_eq!(tag_d.name(), "DD");

        assert_eq!(tag_list.n_items(), 4);

        assert_eq!(tag_list.get_with_name(&tag_a.name()), Some(tag_a.clone()));
        assert_eq!(
            tag_list
                .item(tag_a_index as u32)
                .map(|o| o.downcast::<Tag>().unwrap()),
            Some(tag_a)
        );

        assert_eq!(tag_list.get_with_name(&tag_b.name()), Some(tag_b.clone()));
        assert_eq!(
            tag_list
                .item(tag_b_index as u32)
                .map(|o| o.downcast::<Tag>().unwrap()),
            Some(tag_b)
        );

        assert_eq!(tag_list.get_with_name(&tag_c.name()), Some(tag_c.clone()));
        assert_eq!(
            tag_list
                .item(tag_c_index as u32)
                .map(|o| o.downcast::<Tag>().unwrap()),
            Some(tag_c)
        );

        assert_eq!(tag_list.get_with_name(&tag_d.name()), Some(tag_d.clone()));
        assert_eq!(
            tag_list
                .item(tag_d_index as u32)
                .map(|o| o.downcast::<Tag>().unwrap()),
            Some(tag_d)
        );
    }

    #[test]
    fn rename_tag_with_duplicates() {
        let tag_list = TagList::new();

        let tag_a = Tag::new("A");
        assert!(tag_list.append(tag_a.clone()).is_ok());
        assert_eq!(tag_a.name(), "A");

        let tag_b = Tag::new("B");
        assert!(tag_list.append(tag_b.clone()).is_ok());
        assert_eq!(tag_b.name(), "B");

        assert_eq!(tag_list.n_items(), 2);

        assert!(tag_list.rename_tag(&tag_a, "AA").is_ok());
        assert!(tag_list.rename_tag(&tag_a, "B").is_err());
        assert_eq!(tag_a.name(), "AA");

        assert_eq!(tag_list.n_items(), 2);

        assert!(tag_list.rename_tag(&tag_b, "A").is_ok());
        assert!(tag_list.rename_tag(&tag_b, "AA").is_err());
        assert_eq!(tag_b.name(), "A");

        assert_eq!(tag_list.n_items(), 2);
    }

    #[test]
    fn contains() {
        let tag_list = TagList::new();
        let tag_a = Tag::new("A");
        let tag_a2 = Tag::new("a");
        let tag_b = Tag::new("B");
        let tag_b2 = Tag::new("b");

        assert!(tag_list.append(tag_a.clone()).is_ok());
        assert!(tag_list.append(tag_a2.clone()).is_ok());
        assert!(tag_list.append(tag_b.clone()).is_ok());
        assert!(tag_list.append(tag_b2.clone()).is_ok());

        assert!(tag_list.contains(&tag_a));
        assert!(tag_list.contains(&tag_a2));
        assert!(tag_list.contains(&tag_b));
        assert!(tag_list.contains(&tag_b2));
        assert!(!tag_list.contains(&Tag::new("C")));
        assert!(!tag_list.contains(&Tag::new("c")));
    }

    #[test]
    fn contains_with_name() {
        let tag_list = TagList::new();

        assert!(tag_list.append(Tag::new("A")).is_ok());
        assert!(tag_list.append(Tag::new("a")).is_ok());
        assert!(tag_list.append(Tag::new("B")).is_ok());
        assert!(tag_list.append(Tag::new("b")).is_ok());

        assert!(tag_list.contains_with_name("A"));
        assert!(tag_list.contains_with_name("a"));
        assert!(tag_list.contains_with_name("B"));
        assert!(tag_list.contains_with_name("b"));
        assert!(!tag_list.contains_with_name("C"));
        assert!(!tag_list.contains_with_name("c"));
    }

    #[test]
    fn get_with_name() {
        let tag_list = TagList::new();
        let tag_a = Tag::new("A");
        let tag_a2 = Tag::new("a");
        let tag_b = Tag::new("B");
        let tag_b2 = Tag::new("b");

        assert!(tag_list.append(tag_a.clone()).is_ok());
        assert!(tag_list.append(tag_a2.clone()).is_ok());
        assert!(tag_list.append(tag_b.clone()).is_ok());
        assert!(tag_list.append(tag_b2.clone()).is_ok());

        assert_eq!(tag_list.get_with_name("A"), Some(tag_a));
        assert_eq!(tag_list.get_with_name("a"), Some(tag_a2));
        assert_eq!(tag_list.get_with_name("B"), Some(tag_b));
        assert_eq!(tag_list.get_with_name("b"), Some(tag_b2));
        assert!(tag_list.get_with_name("C").is_none());
        assert!(tag_list.get_with_name("c").is_none());
    }

    #[test]
    fn get_index_of() {
        let tag_list = TagList::new();
        assert!(tag_list.append(Tag::new("A")).is_ok());
        assert!(tag_list.append(Tag::new("B")).is_ok());
        assert!(tag_list.append(Tag::new("C")).is_ok());

        assert_eq!(
            tag_list
                .get_with_name("A")
                .map(|tag| tag_list.get_index_of(&tag)),
            Some(Some(0))
        );
        assert_eq!(
            tag_list
                .get_with_name("B")
                .map(|tag| tag_list.get_index_of(&tag)),
            Some(Some(1))
        );
        assert_eq!(
            tag_list
                .get_with_name("C")
                .map(|tag| tag_list.get_index_of(&tag)),
            Some(Some(2))
        );
    }

    #[test]
    fn is_valid_name_repeating() {
        let tag_list = TagList::new();
        let tag = Tag::new("A");
        assert!(tag_list.is_valid_name("A"));

        assert!(tag_list.append(tag.clone()).is_ok());
        assert!(!tag_list.is_valid_name("A"));

        assert!(tag_list.remove(&tag).is_ok());
        assert!(tag_list.is_valid_name("A"));
    }

    #[test]
    fn is_valid_name_empty() {
        let tag_list = TagList::new();
        assert!(tag_list.is_valid_name("A"));
        assert!(!tag_list.is_valid_name(""));
    }

    #[test]
    fn serialize() {
        let tag_list = TagList::new();
        tag_list.append(Tag::new("A")).unwrap();
        tag_list.append(Tag::new("B")).unwrap();
        tag_list.append(Tag::new("C")).unwrap();

        let string = serde_yaml::to_string(&tag_list).unwrap();
        assert_eq!(string, "---\n- A\n- B\n- C\n");
    }

    #[test]
    fn deserialize() {
        let tag_list: TagList = serde_yaml::from_str("- A\n- B\n- C\n").unwrap();
        assert!(tag_list.contains_with_name("A"));
        assert!(tag_list.contains_with_name("B"));
        assert!(tag_list.contains_with_name("C"));
        assert_eq!(tag_list.n_items(), 3);
    }
}

use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
};
use indexmap::IndexSet;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::cell::RefCell;

use super::tag::Tag;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct TagList {
        pub list: RefCell<IndexSet<Tag>>,
        pub name_list: RefCell<IndexSet<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TagList {
        const NAME: &'static str = "NwtyTagList";
        type Type = super::TagList;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for TagList {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

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
                .map(|o| o.upcast_ref::<glib::Object>())
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
        let imp = imp::TagList::from_instance(self);

        let tag_name = tag.name();

        if tag_name.is_empty() {
            anyhow::bail!("Tag name cannot be empty");
        }

        let is_name_list_appended = {
            let mut name_list = imp.name_list.borrow_mut();
            name_list.insert(tag_name)
        };

        if !is_name_list_appended {
            anyhow::bail!("Cannot append exisiting tag name");
        }

        tag.connect_name_notify(clone!(@weak self as obj => move |tag, _| {
            if let Some(position) = obj.get_index_of(tag) {
                obj.items_changed(position as u32, 1, 1);
            }
        }));

        {
            let mut list = imp.list.borrow_mut();
            assert!(list.insert(tag));
        }

        self.items_changed(self.n_items() - 1, 0, 1);

        Ok(())
    }

    pub fn remove(&self, tag: &Tag) -> anyhow::Result<()> {
        let imp = imp::TagList::from_instance(self);

        let name_list_removed = {
            let mut name_list = imp.name_list.borrow_mut();
            name_list.shift_remove(&tag.name())
        };

        if !name_list_removed {
            anyhow::bail!("Cannot remove tag name that doesnt exist");
        }

        let removed = {
            let mut list = imp.list.borrow_mut();
            list.shift_remove_full(tag)
        };

        if let Some((position, _)) = removed {
            self.items_changed(position as u32, 1, 0);
        } else {
            anyhow::bail!("Cannot remove tag that doesnt exist");
        }

        Ok(())
    }

    pub fn rename_tag(&self, tag: &Tag, name: &str) -> anyhow::Result<()> {
        if self.contains_with_name(name) {
            anyhow::bail!("Cannot rename a tag to a name that already exist");
        }

        if name.is_empty() {
            anyhow::bail!("Tag name cannot be empty");
        }

        let imp = imp::TagList::from_instance(self);
        let previous_name = tag.name();

        {
            let mut name_list = imp.name_list.borrow_mut();
            // Put new name at the end
            assert!(name_list.insert(name.to_string()));
            // Remove the old name at the name_list and replace it with name from the end
            assert!(name_list.swap_remove(&previous_name));
        }

        tag.set_name(name);

        Ok(())
    }

    pub fn contains(&self, tag: &Tag) -> bool {
        let imp = imp::TagList::from_instance(self);
        imp.list.borrow().contains(tag)
    }

    pub fn contains_with_name(&self, name: &str) -> bool {
        let imp = imp::TagList::from_instance(self);
        imp.name_list.borrow().contains(name)
    }

    pub fn get_with_name(&self, name: &str) -> Option<Tag> {
        let imp = imp::TagList::from_instance(self);
        let index = imp.name_list.borrow().get_index_of(name)?;
        imp.list.borrow().get_index(index).cloned()
    }

    pub fn is_valid_name(&self, name: &str) -> bool {
        !self.contains_with_name(name) && !name.is_empty()
    }

    fn get_index_of(&self, tag: &Tag) -> Option<usize> {
        let imp = imp::TagList::from_instance(self);
        imp.list.borrow().get_index_of(tag)
    }
}

// FIXME better ser & de
impl Serialize for TagList {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let imp = imp::TagList::from_instance(self);
        imp.name_list.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TagList {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name_set: Vec<String> = Vec::deserialize(deserializer)?;

        let tag_list = Self::new();
        for name in name_set.iter() {
            let tag = Tag::new(name);
            if let Err(err) = tag_list.append(tag) {
                log::warn!("Error appending a tag, skipping: {}", err);
            }
        }

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
        assert!(tag_list.append(Tag::new("A")).is_ok());
        assert_eq!(tag_list.n_items(), 1);
    }

    #[test]
    fn multiple_append() {
        let tag_list = TagList::new();
        assert!(tag_list.append(Tag::new("A")).is_ok());
        assert_eq!(tag_list.n_items(), 1);
        assert!(tag_list.append(Tag::new("a")).is_ok());
        assert_eq!(tag_list.n_items(), 2);
        assert!(tag_list.append(Tag::new("B")).is_ok());
        assert_eq!(tag_list.n_items(), 3);
        assert!(tag_list.append(Tag::new("b")).is_ok());
        assert_eq!(tag_list.n_items(), 4);
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

        assert!(tag_list.append(tag.clone()).is_ok());
        assert_eq!(tag_list.n_items(), 1);

        assert!(tag_list.remove(&tag).is_ok());
        assert_eq!(tag_list.n_items(), 0);
    }

    #[test]
    fn multiple_remove() {
        let tag_list = TagList::new();
        let tag_a = Tag::new("A");
        let tag_a2 = Tag::new("a");
        let tag_b = Tag::new("B");
        let tag_b2 = Tag::new("b");

        assert!(tag_list.append(tag_a.clone()).is_ok());
        assert!(tag_list.append(tag_a2.clone()).is_ok());
        assert!(tag_list.append(tag_b.clone()).is_ok());
        assert!(tag_list.append(tag_b2.clone()).is_ok());
        assert_eq!(tag_list.n_items(), 4);

        assert!(tag_list.remove(&tag_a).is_ok());
        assert_eq!(tag_list.n_items(), 3);
        assert!(tag_list.remove(&tag_a2).is_ok());
        assert_eq!(tag_list.n_items(), 2);
        assert!(tag_list.remove(&tag_b).is_ok());
        assert_eq!(tag_list.n_items(), 1);
        assert!(tag_list.remove(&tag_b2).is_ok());
        assert_eq!(tag_list.n_items(), 0);
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
}

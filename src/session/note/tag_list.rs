use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};
use indexmap::IndexSet;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::cell::RefCell;

use super::tag::Tag;

// This is used for two different tags with the same name to be treated
// as equal and have the same hash
#[derive(Debug, Serialize, Deserialize, Eq)]
#[serde(transparent)]
pub struct TagWrapper(Tag);

impl std::hash::Hash for TagWrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.name().hash(state);
    }
}

impl PartialEq for TagWrapper {
    fn eq(&self, other: &TagWrapper) -> bool {
        self.0.name() == other.0.name()
    }
}

impl TagWrapper {
    fn inner_ref(&self) -> &Tag {
        &self.0
    }
}

struct TagIdentifier<'a>(&'a str);

impl indexmap::Equivalent<TagWrapper> for TagIdentifier<'_> {
    fn equivalent(&self, key: &TagWrapper) -> bool {
        self.0 == key.inner_ref().name()
    }
}

impl std::hash::Hash for TagIdentifier<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'a> TagIdentifier<'a> {
    pub fn from_str(string: &'a str) -> Self {
        Self(string)
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct TagList {
        pub list: RefCell<IndexSet<TagWrapper>>,
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
                .map(|o| o.inner_ref().upcast_ref::<glib::Object>())
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

    pub fn append(&self, tag: Tag) -> bool {
        let imp = &imp::TagList::from_instance(self);

        let is_appended = {
            let mut list = imp.list.borrow_mut();
            list.insert(TagWrapper(tag))
        };

        if is_appended {
            self.items_changed(self.n_items() - 1, 0, 1);
        }

        is_appended
    }

    // TODO make the methods below take in only a reference of tag
    pub fn remove(&self, tag: Tag) -> bool {
        let imp = &imp::TagList::from_instance(self);

        let removed = {
            let mut list = imp.list.borrow_mut();
            list.shift_remove_full(&TagWrapper(tag))
        };

        if let Some((position, _)) = removed {
            self.items_changed(position as u32, 1, 0);
        }

        removed.is_some()
    }

    pub fn contains(&self, tag: Tag) -> bool {
        let imp = &imp::TagList::from_instance(self);
        imp.list.borrow().contains(&TagWrapper(tag))
    }

    pub fn find_with_name(&self, name: &str) -> Option<Tag> {
        let identifier = TagIdentifier::from_str(name);

        let imp = &imp::TagList::from_instance(self);
        imp.list
            .borrow()
            .get(&identifier)
            .map(TagWrapper::inner_ref)
            .cloned()
    }

    // FIXME remove this
    pub fn dbg(&self) {
        let imp = &imp::TagList::from_instance(self);
        dbg!(imp
            .list
            .borrow()
            .iter()
            .map(|t| t.inner_ref().name())
            .collect::<Vec<String>>());
    }
}

// FIXME better ser & de
impl Serialize for TagList {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let imp = imp::TagList::from_instance(self);
        imp.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TagList {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let deserialized_priv = imp::TagList::deserialize(deserializer)?;
        let deserialized_priv_list = deserialized_priv.list.take();

        let new_tag_list = Self::new();
        let new_tag_list_priv = imp::TagList::from_instance(&new_tag_list);
        new_tag_list_priv.list.replace(deserialized_priv_list);

        Ok(new_tag_list)
    }
}

impl Default for TagList {
    fn default() -> Self {
        Self::new()
    }
}

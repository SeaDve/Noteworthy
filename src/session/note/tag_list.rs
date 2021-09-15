use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::cell::RefCell;

use super::tag::Tag;

mod imp {
    use super::*;

    #[derive(Debug, Default, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct TagList {
        pub list: RefCell<Vec<Tag>>,
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
                .get(position as usize)
                .map(glib::object::Cast::upcast_ref::<glib::Object>)
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

    pub fn append(&self, tag: Tag) {
        let imp = &imp::TagList::from_instance(self);

        {
            let mut list = imp.list.borrow_mut();
            list.push(tag);
        }

        self.items_changed(self.n_items() - 1, 0, 1);
    }

    pub fn remove(&self, index: u32) {
        let imp = &imp::TagList::from_instance(self);

        {
            let mut list = imp.list.borrow_mut();
            list.remove(index as usize);
        }

        self.items_changed(index, 1, 0);
    }

    // FIXME remove this
    pub fn dbg(&self) {
        let imp = &imp::TagList::from_instance(self);
        dbg!(imp
            .list
            .borrow()
            .iter()
            .map(|t| t.name())
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

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

        if self.contains_with_name(&tag_name) {
            anyhow::bail!("Cannot append exisiting tag with same name");
        }

        tag.connect_name_notify(clone!(@weak self as obj => move |tag, _| {
            if let Some(position) = obj.get_index_of(tag) {
                obj.items_changed(position as u32, 1, 1);
            }
        }));

        let is_list_appended = {
            let mut list = imp.list.borrow_mut();
            list.insert(tag)
        };

        let is_name_list_appended = {
            let mut name_list = imp.name_list.borrow_mut();
            name_list.insert(tag_name)
        };

        if is_list_appended || is_name_list_appended {
            self.items_changed(self.n_items() - 1, 0, 1);
        } else {
            anyhow::bail!("Cannot append exisiting tag");
        }

        Ok(())
    }

    pub fn remove(&self, tag: &Tag) -> anyhow::Result<()> {
        let imp = imp::TagList::from_instance(self);

        if self.contains_with_name(&tag.name()) {
            anyhow::bail!("Cannot remove tag name that doesnt exist");
        }

        let removed = {
            let mut list = imp.list.borrow_mut();
            list.shift_remove_full(tag)
        };

        {
            let mut name_list = imp.name_list.borrow_mut();
            debug_assert!(name_list.shift_remove_full(&tag.name()).is_some());
        }

        if let Some((position, _)) = removed {
            self.items_changed(position as u32, 1, 0);
        } else {
            anyhow::bail!("Cannot remove tag that doesnt exist");
        }

        Ok(())
    }

    pub fn rename_tag(&self, tag: &Tag, name: &str) -> anyhow::Result<()> {
        if self.contains_with_name(name) {
            anyhow::bail!("Cannot rename a tag that already exist");
        }

        let imp = imp::TagList::from_instance(self);

        let previous_name = tag.name();

        {
            let mut name_list = imp.name_list.borrow_mut();
            debug_assert!(name_list.insert(name.to_string()));
            debug_assert!(name_list.swap_remove(&previous_name));
        }

        tag.set_name(name);

        Ok(())
    }

    pub fn contains(&self, tag: &Tag) -> bool {
        let imp = imp::TagList::from_instance(self);
        imp.list.borrow().contains(tag)
    }

    pub fn get_with_name(&self, name: &str) -> Option<Tag> {
        let imp = imp::TagList::from_instance(self);
        let index = imp.name_list.borrow().get_index_of(name)?;
        imp.list.borrow().get_index(index).cloned()
    }

    pub fn contains_with_name(&self, name: &str) -> bool {
        let imp = imp::TagList::from_instance(self);
        imp.name_list.borrow().contains(name)
    }

    fn get_index_of(&self, tag: &Tag) -> Option<usize> {
        let imp = imp::TagList::from_instance(self);
        imp.list.borrow().get_index_of(tag)
    }

    // FIXME remove this
    pub fn dbg(&self) {
        let imp = imp::TagList::from_instance(self);
        dbg!(imp
            .list
            .borrow()
            .iter()
            .map(Tag::name)
            .collect::<Vec<String>>());
        dbg!(imp.name_list.borrow());
    }
}

// FIXME better ser & de
impl Serialize for TagList {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let imp = imp::TagList::from_instance(self);
        self.dbg();
        imp.name_list.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TagList {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let string_set: IndexSet<String> = IndexSet::deserialize(deserializer)?;

        let new_tag_list = Self::new();
        let new_tag_list_priv = imp::TagList::from_instance(&new_tag_list);
        new_tag_list_priv
            .list
            .replace(string_set.iter().map(|name| Tag::new(name)).collect());
        new_tag_list_priv.name_list.replace(string_set);

        Ok(new_tag_list)
    }
}

impl Default for TagList {
    fn default() -> Self {
        Self::new()
    }
}

use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
};
use indexmap::IndexSet;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::cell::RefCell;

use super::super::tag::Tag;
use crate::Application;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct NoteTagList {
        pub list: RefCell<IndexSet<Tag>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NoteTagList {
        const NAME: &'static str = "NwtyNoteTagList";
        type Type = super::NoteTagList;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for NoteTagList {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl ListModelImpl for NoteTagList {
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
    pub struct NoteTagList(ObjectSubclass<imp::NoteTagList>)
        @implements gio::ListModel;
}

impl NoteTagList {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create NoteTagList.")
    }

    pub fn append(&self, tag: Tag) -> anyhow::Result<()> {
        let imp = imp::NoteTagList::from_instance(self);

        tag.connect_name_notify(clone!(@weak self as obj => move |tag, _| {
            if let Some(position) = obj.get_index_of(tag) {
                obj.items_changed(position as u32, 1, 1);
            }
        }));

        let is_list_appended = {
            let mut list = imp.list.borrow_mut();
            list.insert(tag)
        };

        if is_list_appended {
            self.items_changed(self.n_items() - 1, 0, 1);
        } else {
            anyhow::bail!("Cannot append exisiting object tag");
        }

        Ok(())
    }

    pub fn remove(&self, tag: &Tag) -> anyhow::Result<()> {
        let imp = imp::NoteTagList::from_instance(self);

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

    pub fn contains(&self, tag: &Tag) -> bool {
        let imp = imp::NoteTagList::from_instance(self);
        imp.list.borrow().contains(tag)
    }

    fn get_index_of(&self, tag: &Tag) -> Option<usize> {
        let imp = imp::NoteTagList::from_instance(self);
        imp.list.borrow().get_index_of(tag)
    }
}

// FIXME better ser & de
impl Serialize for NoteTagList {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let imp = imp::NoteTagList::from_instance(self);
        imp.list.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NoteTagList {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let tag_name_list: Vec<String> = Vec::deserialize(deserializer)?;

        let app = Application::default();
        let tag_list = app.main_window().session().note_manager().tag_list();

        let new_tag_list = NoteTagList::new();
        for name in tag_name_list {
            let new_tag = tag_list.get_with_name(&name).unwrap_or_else(|| {
                log::error!("Tag with name '{}' not found, Creating new instead", &name);
                Tag::new(&name)
            });
            new_tag_list.append(new_tag).unwrap();
        }

        Ok(new_tag_list)
    }
}

impl Default for NoteTagList {
    fn default() -> Self {
        Self::new()
    }
}

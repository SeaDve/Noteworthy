use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
};
use indexmap::IndexSet;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::cell::RefCell;

use super::Attachment;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AttachmentList {
        pub list: RefCell<IndexSet<Attachment>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AttachmentList {
        const NAME: &'static str = "NwtyAttachmentList";
        type Type = super::AttachmentList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for AttachmentList {}

    impl ListModelImpl for AttachmentList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            Attachment::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .get_index(position as usize)
                .map(|a| a.upcast_ref::<glib::Object>())
                .cloned()
        }
    }
}

glib::wrapper! {
    pub struct AttachmentList(ObjectSubclass<imp::AttachmentList>)
        @implements gio::ListModel;
}

impl AttachmentList {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create AttachmentList.")
    }

    pub fn append(&self, attachment: Attachment) -> anyhow::Result<()> {
        attachment.connect_title_notify(clone!(@weak self as obj => move |attachment| {
            if let Some(position) = obj.get_index_of(attachment) {
                obj.items_changed(position as u32, 1, 1);
            }
        }));

        let is_list_appended = self.imp().list.borrow_mut().insert(attachment);

        anyhow::ensure!(is_list_appended, "Cannot append existing object attachment");

        self.items_changed(self.n_items() - 1, 0, 1);

        Ok(())
    }

    pub fn remove(&self, attachment: &Attachment) -> anyhow::Result<()> {
        let removed = self.imp().list.borrow_mut().shift_remove_full(attachment);

        if let Some((position, _)) = removed {
            self.items_changed(position as u32, 1, 0);
        } else {
            anyhow::bail!("Cannot remove attachment that does not exist");
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.imp().list.borrow().is_empty()
    }

    fn get_index_of(&self, attachment: &Attachment) -> Option<usize> {
        self.imp().list.borrow().get_index_of(attachment)
    }
}

impl std::iter::FromIterator<Attachment> for AttachmentList {
    fn from_iter<I: IntoIterator<Item = Attachment>>(iter: I) -> Self {
        let attachment_list = Self::new();

        for attachment in iter {
            if let Err(err) = attachment_list.append(attachment) {
                log::warn!("Error appending an attachment, skipping: {:?}", err);
            }
        }

        attachment_list
    }
}

impl Serialize for AttachmentList {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.imp().list.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AttachmentList {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let attachments: Vec<Attachment> = Vec::deserialize(deserializer)?;

        let attachment_list = attachments.into_iter().collect::<Self>();

        Ok(attachment_list)
    }
}

impl Default for AttachmentList {
    fn default() -> Self {
        Self::new()
    }
}

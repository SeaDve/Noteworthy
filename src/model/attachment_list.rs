use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};
use indexmap::IndexSet;

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
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for AttachmentList {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

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
        let imp = imp::AttachmentList::from_instance(self);

        let is_list_appended = {
            let mut list = imp.list.borrow_mut();
            list.insert(attachment)
        };

        if is_list_appended {
            self.items_changed(self.n_items() - 1, 0, 1);
        } else {
            anyhow::bail!("Cannot append exisiting object attachment");
        }

        Ok(())
    }

    pub fn remove(&self, attachment: &Attachment) -> anyhow::Result<()> {
        let imp = imp::AttachmentList::from_instance(self);

        let removed = {
            let mut list = imp.list.borrow_mut();
            list.shift_remove_full(attachment)
        };

        if let Some((position, _)) = removed {
            self.items_changed(position as u32, 1, 0);
        } else {
            anyhow::bail!("Cannot remove attachment that doesnt exist");
        }

        Ok(())
    }
}

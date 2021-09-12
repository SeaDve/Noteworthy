use gettextrs::gettext;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use super::{category::Category, Note, NoteList};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ItemList {
        pub list: OnceCell<[glib::Object; 2]>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ItemList {
        const NAME: &'static str = "NwtyItemList";
        type Type = super::ItemList;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for ItemList {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "note-list",
                    "Note list",
                    "Data model for the categories",
                    NoteList::static_type(),
                    glib::ParamFlags::WRITABLE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "note-list" => {
                    let note_list = value.get().unwrap();
                    obj.set_note_list(&note_list);
                }
                _ => unimplemented!(),
            }
        }
    }

    impl ListModelImpl for ItemList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            glib::Object::static_type()
        }
        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.list.get().map_or(0, |l| l.len() as u32)
        }
        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.list
                .get()
                .and_then(|l| l.get(position as usize))
                .map(glib::object::Cast::upcast_ref::<glib::Object>)
                .cloned()
        }
    }
}

glib::wrapper! {
    pub struct ItemList(ObjectSubclass<imp::ItemList>)
        @implements gio::ListModel;
}

impl ItemList {
    pub fn new(note_list: &NoteList) -> Self {
        glib::Object::new(&[("note-list", note_list)]).expect("Failed to create ItemList")
    }

    fn set_note_list(&self, note_list: &NoteList) {
        let imp = imp::ItemList::from_instance(self);

        let list = [
            Category::new(
                &gettext("Pinned"),
                &gtk::CustomFilter::new(|item| {
                    let note = item.downcast_ref::<Note>().unwrap().metadata();
                    note.is_pinned()
                })
                .upcast(),
                note_list,
            )
            .upcast::<glib::Object>(),
            Category::new(
                &gettext("Other Notes"),
                &gtk::CustomFilter::new(|item| {
                    let note = item.downcast_ref::<Note>().unwrap().metadata();
                    !note.is_pinned()
                })
                .upcast(),
                note_list,
            )
            .upcast::<glib::Object>(),
        ];
        let len = list.len() as u32;

        imp.list.set(list).unwrap();
        self.items_changed(0, 0, len);
    }
}

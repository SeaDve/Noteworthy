use gettextrs::gettext;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use once_cell::sync::Lazy;
use once_cell::unsync::OnceCell;

use super::{Item, ItemKind, TagList};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ItemList {
        pub list: OnceCell<[glib::Object; 5]>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ItemList {
        const NAME: &'static str = "NwtyViewSwitcherItemList";
        type Type = super::ItemList;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for ItemList {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "tag-list",
                    "Tag list",
                    "Data model for the categories",
                    TagList::static_type(),
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
                "tag-list" => {
                    let tag_list = value.get().unwrap();
                    obj.set_tag_list(tag_list);
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
    pub fn new(tag_list: &TagList) -> Self {
        glib::Object::new(&[("tag-list", tag_list)]).expect("Failed to create ItemList")
    }

    fn set_tag_list(&self, tag_list: TagList) {
        let imp = imp::ItemList::from_instance(self);

        let list = [
            Item::new(
                ItemKind::AllNotes,
                Some(gettext("All Notes")),
                None::<TagList>,
            )
            .upcast(),
            Item::new(ItemKind::Separator, None, None::<TagList>).upcast(),
            Item::new(ItemKind::Category, Some(gettext("Tags")), Some(tag_list)).upcast(),
            Item::new(ItemKind::Separator, None, None::<TagList>).upcast(),
            Item::new(ItemKind::Trash, Some(gettext("Trash")), None::<TagList>).upcast(),
        ];
        let len = list.len() as u32;

        imp.list.set(list).unwrap();
        self.items_changed(0, 0, len);
    }
}

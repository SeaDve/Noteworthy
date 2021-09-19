use adw::subclass::prelude::*;
use gtk::{
    glib::{self, GBoxed},
    prelude::*,
};

use std::cell::RefCell;

#[derive(Debug, Clone, GBoxed, PartialEq)]
#[gboxed(type_name = "NwtySidebarViewSwitcherItemType")]
pub enum ItemType {
    Separator,
    AllNotes,
    Trash,
}

impl Default for ItemType {
    fn default() -> Self {
        Self::AllNotes
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Item {
        item_type: RefCell<ItemType>,
        display_name: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Item {
        const NAME: &'static str = "NwtySidebarViewSwitcherItem";
        type Type = super::Item;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Item {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_boxed(
                        "item-type",
                        "Item Type",
                        "Type of this item",
                        ItemType::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_string(
                        "display-name",
                        "Display Name",
                        "Display name of this item",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "item-type" => {
                    let item_type = value.get().unwrap();
                    self.item_type.replace(item_type);
                }
                "display-name" => {
                    let display_name = value.get().unwrap();
                    self.display_name.replace(display_name);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "item-type" => self.item_type.borrow().to_value(),
                "display-name" => self.display_name.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Item(ObjectSubclass<imp::Item>);
}

impl Item {
    pub fn new(item_type: ItemType, display_name: Option<String>) -> Self {
        glib::Object::new(&[("item-type", &item_type), ("display-name", &display_name)])
            .expect("Failed to create Item.")
    }

    pub fn item_type(&self) -> ItemType {
        self.property("item-type").unwrap().get().unwrap()
    }

    pub fn display_name(&self) -> Option<String> {
        self.property("display-name").unwrap().get().unwrap()
    }
}

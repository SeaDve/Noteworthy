use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone, GBoxed},
    prelude::*,
};

use std::cell::RefCell;

use crate::session::note::{Tag, TagList};

#[derive(Debug, Clone, GBoxed, PartialEq)]
#[gboxed(type_name = "NwtySidebarViewSwitcherType")]
pub enum ItemKind {
    Separator,
    Category,
    AllNotes,
    Tag(Tag),
    Trash,
}

impl Default for ItemKind {
    fn default() -> Self {
        Self::AllNotes
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Item {
        item_kind: RefCell<ItemKind>,
        display_name: RefCell<Option<String>>,
        model: RefCell<Option<gio::ListModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Item {
        const NAME: &'static str = "NwtySidebarViewSwitcherItem";
        type Type = super::Item;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for Item {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if let Some(ref model) = *self.model.borrow() {
                model.connect_items_changed(clone!(@weak obj => move |_, pos, added, removed| {
                    obj.items_changed(pos, added, removed);
                }));
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_boxed(
                        "item-kind",
                        "Item Kind",
                        "Kind of this item",
                        ItemKind::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_string(
                        "display-name",
                        "Display Name",
                        "Display name of this item",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "model",
                        "Model",
                        "The model of this item",
                        gio::ListModel::static_type(),
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
                "item-kind" => {
                    let item_kind = value.get().unwrap();
                    self.item_kind.replace(item_kind);
                }
                "display-name" => {
                    let display_name = value.get().unwrap();
                    self.display_name.replace(display_name);
                }
                "model" => {
                    let model = value.get().unwrap();
                    self.model.replace(model);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "item-kind" => self.item_kind.borrow().to_value(),
                "display-name" => self.display_name.borrow().to_value(),
                "model" => self.model.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl ListModelImpl for Item {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            Tag::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.model.borrow().as_ref().map_or(0, |l| l.n_items())
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.model.borrow().as_ref().and_then(|l| l.item(position))
        }
    }
}

glib::wrapper! {
    pub struct Item(ObjectSubclass<imp::Item>)
        @implements gio::ListModel;
}

impl Item {
    pub fn new(item_kind: ItemKind, display_name: Option<String>, model: Option<TagList>) -> Self {
        glib::Object::new(&[
            ("item-kind", &item_kind),
            ("display-name", &display_name),
            ("model", &model),
        ])
        .expect("Failed to create Item.")
    }

    pub fn item_kind(&self) -> ItemKind {
        self.property("item-kind").unwrap().get().unwrap()
    }

    pub fn display_name(&self) -> Option<String> {
        self.property("display-name").unwrap().get().unwrap()
    }
}

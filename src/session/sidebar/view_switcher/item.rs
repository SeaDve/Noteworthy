use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, GBoxed},
    prelude::*,
};

use std::cell::RefCell;

use super::Tag;

#[derive(Debug, Clone, GBoxed, PartialEq)]
#[gboxed(type_name = "NwtySidebarViewSwitcherType")]
pub enum ItemKind {
    Separator,
    Category,
    AllNotes,
    EditTags,
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
        kind: RefCell<ItemKind>,
        display_name: RefCell<Option<String>>,
        model: RefCell<Option<gio::ListModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Item {
        const NAME: &'static str = "NwtySidebarViewSwitcherItem";
        type Type = super::Item;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Item {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_boxed(
                        "kind",
                        "Kind",
                        "Kind of this item",
                        ItemKind::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_string(
                        "display-name",
                        "Display Name",
                        "Display name of this item",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
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
                "kind" => {
                    let kind = value.get().unwrap();
                    self.kind.replace(kind);
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
                "kind" => self.kind.borrow().to_value(),
                "display-name" => self.display_name.borrow().to_value(),
                "model" => self.model.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Item(ObjectSubclass<imp::Item>);
}

impl Item {
    pub fn new(
        kind: ItemKind,
        display_name: Option<&str>,
        model: Option<&impl IsA<gio::ListModel>>,
    ) -> Self {
        glib::Object::new(&[
            ("kind", &kind),
            ("display-name", &display_name),
            ("model", &model),
        ])
        .expect("Failed to create Item.")
    }

    pub fn kind(&self) -> ItemKind {
        self.property("kind").unwrap().get().unwrap()
    }

    pub fn display_name(&self) -> Option<String> {
        self.property("display-name").unwrap().get().unwrap()
    }

    pub fn model(&self) -> Option<gio::ListModel> {
        self.property("model").unwrap().get().unwrap()
    }
}

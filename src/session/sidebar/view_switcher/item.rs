use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};

use std::cell::RefCell;

use super::ItemKind;

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
    pub const fn builder(kind: ItemKind) -> ItemBuilder {
        ItemBuilder::new(kind)
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

pub struct ItemBuilder {
    kind: ItemKind,
    display_name: Option<String>,
    model: Option<gio::ListModel>,
}

impl ItemBuilder {
    pub const fn new(kind: ItemKind) -> Self {
        Self {
            kind,
            display_name: None,
            model: None,
        }
    }

    pub fn display_name(mut self, display_name: &str) -> Self {
        self.display_name = Some(display_name.to_string());
        self
    }

    pub fn model(mut self, model: &impl IsA<gio::ListModel>) -> Self {
        self.model = Some(model.clone().upcast());
        self
    }

    pub fn build(self) -> Item {
        let mut properties: Vec<(&str, &dyn ToValue)> = vec![("kind", &self.kind)];

        if let Some(ref display_name) = self.display_name {
            properties.push(("display-name", display_name));
        }

        if let Some(ref model) = self.model {
            properties.push(("model", model));
        }

        glib::Object::new::<Item>(&properties).expect("Failed to create an instance of Item")
    }
}

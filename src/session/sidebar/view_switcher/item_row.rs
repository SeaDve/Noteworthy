use adw::{prelude::*, subclass::prelude::BinImpl};
use gtk::{glib, subclass::prelude::*, CompositeTemplate};

use std::cell::{Cell, RefCell};

use super::{Item, ItemKind, Tag};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sidebar-view-switcher-item-row.ui")]
    pub struct ItemRow {
        #[template_child]
        pub label_child: TemplateChild<gtk::Label>,
        #[template_child]
        pub separator_child: TemplateChild<gtk::Separator>,
        #[template_child]
        pub category_child: TemplateChild<gtk::Label>,
        #[template_child]
        pub edit_tags_child: TemplateChild<gtk::Button>,
        #[template_child]
        pub bin: TemplateChild<adw::Bin>,
        #[template_child]
        pub select_icon: TemplateChild<gtk::Image>,

        pub item: RefCell<Option<Item>>,
        pub selected: Cell<bool>,
        pub list_row: RefCell<Option<gtk::TreeListRow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ItemRow {
        const NAME: &'static str = "NwtySidebarViewSwitcherItemRow";
        type Type = super::ItemRow;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ItemRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "item",
                        "Item",
                        "The item being represented by this",
                        Item::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_boolean(
                        "selected",
                        "Selected",
                        "Whether this row is selected",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_object(
                        "list-row",
                        "List Row",
                        "The list row to track for expander state",
                        gtk::TreeListRow::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                ]
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
                "selected" => {
                    let selected = value.get().unwrap();
                    obj.set_selected(selected);
                }
                "list-row" => {
                    let list_row = value.get().unwrap();
                    obj.set_list_row(list_row);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "item" => obj.item().to_value(),
                "selected" => obj.selected().to_value(),
                "list-row" => obj.list_row().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ItemRow {}
    impl BinImpl for ItemRow {}
}

glib::wrapper! {
    pub struct ItemRow(ObjectSubclass<imp::ItemRow>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl ItemRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ItemRow")
    }

    pub fn selected(&self) -> bool {
        let imp = imp::ItemRow::from_instance(self);
        imp.selected.get()
    }

    pub fn set_selected(&self, selected: bool) {
        let imp = imp::ItemRow::from_instance(self);
        imp.select_icon.set_visible(selected);

        imp.selected.set(selected);
        self.notify("selected");
    }

    pub fn item(&self) -> Option<glib::Object> {
        self.list_row().and_then(|r| r.item())
    }

    pub fn list_row(&self) -> Option<gtk::TreeListRow> {
        let imp = imp::ItemRow::from_instance(self);
        imp.list_row.borrow().clone()
    }

    pub fn set_list_row(&self, list_row: Option<gtk::TreeListRow>) {
        let imp = imp::ItemRow::from_instance(self);

        if self.list_row() == list_row {
            return;
        }

        if list_row.is_some() {
            imp.list_row.replace(list_row);
        } else {
            return;
        }

        if let Some(item) = self.item() {
            if let Some(item) = item.downcast_ref::<Item>() {
                match item.item_kind() {
                    ItemKind::AllNotes | ItemKind::Trash => {
                        imp.label_child.set_label(&item.display_name().unwrap());
                        imp.bin.set_child(Some(&imp.label_child.get()));
                        self.set_margin_start(6);
                        self.set_margin_end(6);
                    }
                    ItemKind::Category => {
                        imp.category_child.set_label(&item.display_name().unwrap());
                        imp.bin.set_child(Some(&imp.category_child.get()));
                        self.set_margin_start(6);
                        self.set_margin_end(6);
                    }
                    ItemKind::EditTags => {
                        imp.bin.set_child(Some(&imp.edit_tags_child.get()));
                        self.set_margin_start(6);
                        self.set_margin_end(6);
                    }
                    ItemKind::Separator => {
                        imp.bin.set_child(Some(&imp.separator_child.get()));
                        self.set_margin_start(0);
                        self.set_margin_end(0);
                    }
                    ItemKind::Tag(_) => unreachable!("This is handled by below"),
                }
            } else if let Some(tag) = item.downcast_ref::<Tag>() {
                imp.label_child.set_label(&tag.name());
                imp.bin.set_child(Some(&imp.label_child.get()));
                self.set_margin_start(6);
                self.set_margin_end(6);
            } else {
                panic!("Wrong row item: {:?}", item);
            }
        }

        self.notify("item");
        self.notify("list-row");
    }
}

use gtk::{glib, prelude::*, subclass::prelude::*};

use std::cell::{Cell, RefCell};

use super::{Item, ItemKind, Tag};

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

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
        pub select_icon: TemplateChild<gtk::Image>,

        pub binding: RefCell<Option<glib::Binding>>,

        pub item: RefCell<Option<Item>>,
        pub selected: Cell<bool>,
        pub list_row: RefCell<Option<gtk::TreeListRow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ItemRow {
        const NAME: &'static str = "NwtySidebarViewSwitcherItemRow";
        type Type = super::ItemRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_css_name("itemrow");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ItemRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "item",
                        "Item",
                        "The item being represented by this",
                        Item::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "selected",
                        "Selected",
                        "Whether this row is selected",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
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

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for ItemRow {}
}

glib::wrapper! {
    pub struct ItemRow(ObjectSubclass<imp::ItemRow>)
        @extends gtk::Widget;
}

impl ItemRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ItemRow")
    }

    pub fn selected(&self) -> bool {
        self.imp().selected.get()
    }

    pub fn set_selected(&self, selected: bool) {
        let imp = self.imp();
        imp.select_icon.set_visible(selected);
        imp.selected.set(selected);
        self.notify("selected");
    }

    pub fn item(&self) -> Option<glib::Object> {
        self.list_row().and_then(|r| r.item())
    }

    pub fn list_row(&self) -> Option<gtk::TreeListRow> {
        self.imp().list_row.borrow().clone()
    }

    pub fn set_list_row(&self, list_row: Option<gtk::TreeListRow>) {
        let imp = self.imp();

        if self.list_row() == list_row {
            return;
        }

        if list_row.is_some() {
            imp.list_row.replace(list_row);
        } else {
            return;
        }

        if let Some(binding) = imp.binding.take() {
            binding.unbind();
        }

        // FIXME use cleaner approach, so each row does have an own widget
        // not handled by this object
        if let Some(item) = self.item() {
            if let Some(item) = item.downcast_ref::<Item>() {
                match item.kind() {
                    ItemKind::AllNotes | ItemKind::Trash => {
                        imp.label_child.set_label(&item.display_name().unwrap());
                        self.insert_before_select_icon(&imp.label_child.get());
                    }
                    ItemKind::Category => {
                        imp.category_child.set_label(&item.display_name().unwrap());
                        self.insert_before_select_icon(&imp.category_child.get());
                    }
                    ItemKind::EditTags => {
                        self.insert_before_select_icon(&imp.edit_tags_child.get());
                    }
                    ItemKind::Separator => {
                        self.insert_before_select_icon(&imp.separator_child.get());
                    }
                    ItemKind::Tag(_) => unreachable!("This is handled by below"),
                }
            } else if let Some(tag) = item.downcast_ref::<Tag>() {
                let binding = tag
                    .bind_property("name", &imp.label_child.get(), "label")
                    .flags(glib::BindingFlags::SYNC_CREATE)
                    .build();
                imp.binding.replace(Some(binding));
                self.insert_before_select_icon(&imp.label_child.get());
            } else {
                unreachable!("Invalid row item `{:?}`", item);
            }
        }

        self.notify("item");
        self.notify("list-row");
    }

    fn insert_before_select_icon(&self, widget: &impl IsA<gtk::Widget>) {
        widget.insert_before(self, Some(&self.imp().select_icon.get()));
    }
}

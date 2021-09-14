use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use std::cell::RefCell;

use super::{Item, ItemRow, ItemType};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sidebar-view-switcher-popover.ui")]
    pub struct Popover {
        #[template_child]
        listview: TemplateChild<gtk::ListView>,

        selected_item: RefCell<Option<Item>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Popover {
        const NAME: &'static str = "NwtySidebarViewSwitcherPopover";
        type Type = super::Popover;
        type ParentType = gtk::Popover;

        fn class_init(klass: &mut Self::Class) {
            ItemRow::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Popover {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "selected-item",
                    "Selected-item",
                    "The selected item in popover",
                    Item::static_type(),
                    glib::ParamFlags::READWRITE,
                )]
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
                "selected-item" => {
                    let selected_item = value.get().unwrap();
                    self.selected_item.replace(selected_item);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-item" => self.selected_item.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_, list_item| {
                let item_row = ItemRow::new();

                let list_item_expression = gtk::ConstantExpression::new(list_item);

                let item_expression = gtk::PropertyExpression::new(
                    gtk::ListItem::static_type(),
                    Some(&list_item_expression),
                    "item",
                );
                item_expression.bind(&item_row, "item", None);

                let selected_expression = gtk::PropertyExpression::new(
                    gtk::ListItem::static_type(),
                    Some(&list_item_expression),
                    "selected",
                );
                selected_expression.bind(&item_row, "selected", None);

                list_item.set_child(Some(&item_row));
            });

            factory.connect_bind(|_, list_item| {
                let item: Item = list_item.item().unwrap().downcast().unwrap();

                if item.item_type() == ItemType::Separator {
                    list_item.set_selectable(false);
                }
            });

            self.listview.set_factory(Some(&factory));

            let model = gio::ListStore::new(Item::static_type());
            model.append(&Item::new(ItemType::AllNotes, Some(gettext("All Notes"))));
            model.append(&Item::new(ItemType::Separator, None));
            model.append(&Item::new(ItemType::Trash, Some(gettext("Trash"))));

            let selection_model = gtk::SingleSelection::new(Some(&model));
            selection_model
                .bind_property("selected-item", obj, "selected-item")
                .transform_to(|_, value| {
                    let selected_item: Option<Item> = value
                        .get::<Option<glib::Object>>()
                        .unwrap()
                        .map(|si| si.downcast().unwrap());
                    Some(selected_item.to_value())
                })
                .flags(glib::BindingFlags::SYNC_CREATE)
                .build();

            self.listview.set_model(Some(&selection_model));
        }
    }

    impl WidgetImpl for Popover {}
    impl PopoverImpl for Popover {}
}

glib::wrapper! {
    pub struct Popover(ObjectSubclass<imp::Popover>)
        @extends gtk::Widget, gtk::Popover,
        @implements gtk::Accessible;
}

impl Popover {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Popover.")
    }

    pub fn selected_item(&self) -> Option<Item> {
        self.property("selected-item").unwrap().get().unwrap()
    }
}

mod item;
mod item_list;
mod item_row;
mod popover;

use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use std::cell::RefCell;

pub use self::item::Type;
use self::{item::Item, item_row::ItemRow, popover::Popover};
use crate::session::note::TagList;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sidebar-view-switcher.ui")]
    pub struct ViewSwitcher {
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub popover: TemplateChild<Popover>,

        pub selected_type: RefCell<Type>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ViewSwitcher {
        const NAME: &'static str = "NwtySidebarViewSwitcher";
        type Type = super::ViewSwitcher;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            ItemRow::static_type();
            Popover::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ViewSwitcher {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_boxed(
                    "selected-type",
                    "Selected-type",
                    "The selected type in the switcher",
                    Type::static_type(),
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
                "selected-type" => {
                    let selected_type = value.get().unwrap();
                    self.selected_type.replace(selected_type);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-type" => self.selected_type.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let popover_expression = gtk::ConstantExpression::new(&self.popover.get());
            let selected_item_expression = gtk::PropertyExpression::new(
                Popover::static_type(),
                Some(&popover_expression),
                "selected-item",
            );
            let label_expression = gtk::PropertyExpression::new(
                Item::static_type(),
                Some(&selected_item_expression),
                "display-name",
            );
            label_expression.bind(&self.menu_button.get(), "label", None);

            let selected_type_expression = gtk::PropertyExpression::new(
                Item::static_type(),
                Some(&selected_item_expression),
                "type",
            );
            selected_type_expression.bind(obj, "selected-type", None);
        }
    }

    impl WidgetImpl for ViewSwitcher {}
    impl BinImpl for ViewSwitcher {}
}

glib::wrapper! {
    pub struct ViewSwitcher(ObjectSubclass<imp::ViewSwitcher>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl ViewSwitcher {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ViewSwitcher.")
    }

    pub fn selected_type(&self) -> Type {
        self.property("selected-type").unwrap().get().unwrap()
    }

    pub fn connect_selected_type_notify<F: Fn(&Self, &glib::ParamSpec) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_notify_local(Some("selected-type"), f)
    }

    pub fn set_tag_list(&self, tag_list: TagList) {
        let imp = imp::ViewSwitcher::from_instance(self);
        imp.popover.set_tag_list(tag_list);
    }
}

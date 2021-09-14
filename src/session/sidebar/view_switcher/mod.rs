mod item;
mod item_row;
mod popover;

use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use self::{
    item::{Item, ItemType},
    item_row::ItemRow,
    popover::Popover,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sidebar-view-switcher.ui")]
    pub struct ViewSwitcher {
        #[template_child]
        menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        popover: TemplateChild<Popover>,
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let popover_expression = gtk::ConstantExpression::new(&self.popover.get());
            let selected_item_expression = gtk::PropertyExpression::new(
                Popover::static_type(),
                Some(&popover_expression),
                "selected-item",
            );
            let label_expression = gtk::ClosureExpression::new(
                |args| {
                    let item: Option<Item> = args[1].get().unwrap();
                    item.unwrap()
                        .display_name()
                        .expect("Separator can't have a display name")
                },
                &[selected_item_expression.upcast()],
            );

            label_expression.bind(&self.menu_button.get(), "label", None);
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
}

mod item;
mod item_row;

use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use self::{
    item::{Item, ItemType},
    item_row::ItemRow,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sidebar-view-switcher.ui")]
    pub struct ViewSwitcher {
        #[template_child]
        listview: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ViewSwitcher {
        const NAME: &'static str = "NwtySidebarViewSwitcher";
        type Type = super::ViewSwitcher;
        type ParentType = gtk::Popover;

        fn class_init(klass: &mut Self::Class) {
            ItemRow::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ViewSwitcher {
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
            model.append(&Item::new(ItemType::AllNotes));
            model.append(&Item::new(ItemType::Separator));
            model.append(&Item::new(ItemType::Trash));

            let selection_model = gtk::SingleSelection::new(Some(&model));
            self.listview.set_model(Some(&selection_model));
        }
    }

    impl WidgetImpl for ViewSwitcher {}
    impl PopoverImpl for ViewSwitcher {}
}

glib::wrapper! {
    pub struct ViewSwitcher(ObjectSubclass<imp::ViewSwitcher>)
        @extends gtk::Widget, gtk::Popover,
        @implements gtk::Accessible;
}

impl ViewSwitcher {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ViewSwitcher.")
    }
}

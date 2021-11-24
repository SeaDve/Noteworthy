mod item;
mod item_kind;
mod item_row;

use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use std::cell::RefCell;

pub use self::item_kind::ItemKind;
use self::{item::Item, item_row::ItemRow};
use crate::{
    model::{Tag, TagList},
    utils::{ChainExpr, PropExpr},
};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sidebar-view-switcher.ui")]
    pub struct ViewSwitcher {
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub list_view: TemplateChild<gtk::ListView>,

        pub selected_item: RefCell<Option<glib::Object>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ViewSwitcher {
        const NAME: &'static str = "NwtySidebarViewSwitcher";
        type Type = super::ViewSwitcher;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            ItemRow::static_type();
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
                vec![
                    glib::ParamSpec::new_object(
                        "selected-item",
                        "Selected-item",
                        "The selected item in popover",
                        glib::Object::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_boxed(
                        "selected-type",
                        "Selected-type",
                        "The selected type in the switcher",
                        ItemKind::static_type(),
                        glib::ParamFlags::READABLE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_object(
                        "tag-list",
                        "Tag List",
                        "The tag list in the view switcher",
                        TagList::static_type(),
                        glib::ParamFlags::WRITABLE | glib::ParamFlags::EXPLICIT_NOTIFY,
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
                "selected-item" => {
                    let selected_item = value.get().unwrap();
                    obj.set_selected_item(selected_item);
                }
                "tag-list" => {
                    let tag_list = value.get().unwrap();
                    obj.set_tag_list(tag_list);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-item" => obj.selected_item().to_value(),
                "selected-type" => obj.selected_type().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_list_view();
            obj.setup_expressions();
        }
    }

    impl WidgetImpl for ViewSwitcher {}
    impl BinImpl for ViewSwitcher {}
}

glib::wrapper! {
    pub struct ViewSwitcher(ObjectSubclass<imp::ViewSwitcher>)
        @extends gtk::Widget, adw::Bin;
}

impl ViewSwitcher {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ViewSwitcher.")
    }

    pub fn set_tag_list(&self, tag_list: TagList) {
        let imp = imp::ViewSwitcher::from_instance(self);

        let items = &[
            Item::builder(ItemKind::AllNotes)
                .display_name(&gettext("All Notes"))
                .build()
                .upcast(),
            Item::builder(ItemKind::Separator).build().upcast(),
            Item::builder(ItemKind::Category)
                .display_name(&gettext("Tags"))
                .model(&tag_list)
                .build()
                .upcast(),
            Item::builder(ItemKind::EditTags).build().upcast(),
            Item::builder(ItemKind::Separator).build().upcast(),
            Item::builder(ItemKind::Trash)
                .display_name(&gettext("Trash"))
                .build()
                .upcast(),
        ];
        let item_list = gio::ListStore::new(Item::static_type());
        item_list.splice(0, 0, items);

        let tree_model = gtk::TreeListModel::new(&item_list, false, true, |item| {
            item.downcast_ref::<Item>().and_then(|item| item.model())
        });

        let selection_model = gtk::SingleSelection::new(Some(&tree_model));
        selection_model
            .bind_property("selected-item", self, "selected-item")
            .transform_to(|_, value| {
                value
                    .get::<Option<glib::Object>>()
                    .unwrap()
                    .map(|o| o.downcast::<gtk::TreeListRow>().unwrap().item().unwrap())
                    .map(|i| i.to_value())
            })
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();

        imp.list_view.set_model(Some(&selection_model));
        self.notify("tag-list");
    }

    pub fn connect_selected_type_notify<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_notify_local(Some("selected-type"), move |obj, _| f(obj))
    }

    pub fn selected_type(&self) -> ItemKind {
        self.selected_item()
            .map_or(ItemKind::AllNotes, |selected_item| {
                if let Some(item) = selected_item.downcast_ref::<Item>() {
                    match item.kind() {
                        ItemKind::Separator | ItemKind::Category | ItemKind::EditTags => {
                            let imp = imp::ViewSwitcher::from_instance(self);
                            let model: gtk::SingleSelection =
                                imp.list_view.model().unwrap().downcast().unwrap();
                            // These three get selected when trying to delete an item that was selected.
                            // Therefore, select the first item, AllNotes, instead. Maybe a GTK bug?
                            model.set_selected(0);
                            ItemKind::AllNotes
                        }
                        other_kind => other_kind,
                    }
                } else if let Some(tag) = selected_item.downcast_ref::<Tag>() {
                    ItemKind::Tag(tag.clone())
                } else {
                    unreachable!("Invalid selected item {:?}", selected_item);
                }
            })
    }

    fn set_selected_item(&self, selected_item: Option<glib::Object>) {
        let imp = imp::ViewSwitcher::from_instance(self);
        imp.selected_item.replace(selected_item);
        self.notify("selected-item");
        self.notify("selected-type");
    }

    fn selected_item(&self) -> Option<glib::Object> {
        let imp = imp::ViewSwitcher::from_instance(self);
        imp.selected_item.borrow().clone()
    }

    fn setup_expressions(&self) {
        let imp = imp::ViewSwitcher::from_instance(self);

        self.property_expression("selected-item")
            .closure_expression(|args| {
                let selected_item: Option<glib::Object> = args[1].get().unwrap();

                selected_item.map_or(String::new(), |selected_item| {
                    if let Some(tag) = selected_item.downcast_ref::<Tag>() {
                        tag.name()
                    } else if let Some(item) = selected_item.downcast_ref::<Item>() {
                        item.display_name().unwrap()
                    } else {
                        unreachable!("Invalid selected item {:?}", selected_item);
                    }
                })
            })
            .bind(&imp.menu_button.get(), "label", None::<&gtk::Widget>);
    }

    fn setup_list_view(&self) {
        let imp = imp::ViewSwitcher::from_instance(self);

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_, list_item| {
            let item_row = ItemRow::new();

            list_item
                .property_expression("item")
                .bind(&item_row, "list-row", None::<&gtk::Widget>);

            list_item.property_expression("selected").bind(
                &item_row,
                "selected",
                None::<&gtk::Widget>,
            );

            list_item.set_child(Some(&item_row));
        });

        factory.connect_bind(|_, list_item| {
            let item: Option<Item> = list_item
                .item()
                .unwrap()
                .downcast::<gtk::TreeListRow>()
                .unwrap()
                .item()
                .and_then(|o| o.downcast().ok());

            if let Some(item) = item {
                match item.kind() {
                    ItemKind::Separator | ItemKind::Category | ItemKind::EditTags => {
                        list_item.set_selectable(false);
                    }
                    _ => (),
                }
            }
        });

        imp.list_view.set_factory(Some(&factory));

        // FIXME popdown this popover when something is clicked
    }
}

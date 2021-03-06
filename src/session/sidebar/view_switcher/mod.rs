mod item;
mod item_kind;
mod item_row;

use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{
    gio,
    glib::{self, closure},
    prelude::*,
    subclass::prelude::*,
};

use std::cell::RefCell;

pub use self::item_kind::ItemKind;
use self::{item::Item, item_row::ItemRow};
use crate::model::{Tag, TagList};

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

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
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ViewSwitcher {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "selected-item",
                        "Selected-item",
                        "The selected item in popover",
                        glib::Object::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "selected-type",
                        "Selected-type",
                        "The selected type in the switcher",
                        ItemKind::static_type(),
                        glib::ParamFlags::READABLE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecObject::new(
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
                    obj.set_tag_list(&tag_list);
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

    pub fn set_tag_list(&self, tag_list: &TagList) {
        let items: &[glib::Object; 6] = &[
            Item::builder(ItemKind::AllNotes)
                .display_name(&gettext("All Notes"))
                .build()
                .upcast(),
            Item::builder(ItemKind::Separator).build().upcast(),
            Item::builder(ItemKind::Category)
                .display_name(&gettext("Tags"))
                .model(tag_list)
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

        self.imp().list_view.set_model(Some(&selection_model));
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
                            let model: gtk::SingleSelection =
                                self.imp().list_view.model().unwrap().downcast().unwrap();
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
                    unreachable!("Invalid selected item `{:?}`", selected_item);
                }
            })
    }

    fn set_selected_item(&self, selected_item: Option<glib::Object>) {
        self.imp().selected_item.replace(selected_item);
        self.notify("selected-item");
        self.notify("selected-type");
    }

    fn selected_item(&self) -> Option<glib::Object> {
        self.imp().selected_item.borrow().clone()
    }

    fn setup_expressions(&self) {
        Self::this_expression("selected-item")
            .chain_closure::<String>(closure!(|_: Self, selected_item: Option<glib::Object>| {
                selected_item.map_or(String::new(), |selected_item| {
                    if let Some(tag) = selected_item.downcast_ref::<Tag>() {
                        tag.name()
                    } else if let Some(item) = selected_item.downcast_ref::<Item>() {
                        // FIXME The selected item is set to `EditTags` temporarily
                        // which doesn't have `display_name`, panicking.
                        // A solution is to handle `Separator`, `Category`, and `EditTags` being
                        // selected directly from the binding's `transform_to`, instead of handling
                        // it when `selected_type` is called. When that is changed, replace too
                        // `unwrap_or_default` to just `unwrap` or `expect`.
                        item.display_name().unwrap_or_default()
                    } else {
                        unreachable!("Invalid selected item `{:?}`", selected_item);
                    }
                })
            }))
            .bind(&self.imp().menu_button.get(), "label", Some(self));
    }

    fn setup_list_view(&self) {
        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_, list_item| {
            let item_row = ItemRow::new();

            list_item
                .property_expression("item")
                .bind(&item_row, "list-row", glib::Object::NONE);

            list_item.property_expression("selected").bind(
                &item_row,
                "selected",
                glib::Object::NONE,
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
                    ItemKind::AllNotes | ItemKind::Tag(_) | ItemKind::Trash => (),
                }
            }
        });

        self.imp().list_view.set_factory(Some(&factory));

        // FIXME popdown this popover when something is clicked
    }
}

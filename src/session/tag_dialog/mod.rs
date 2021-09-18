mod row;

use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};
use once_cell::unsync::OnceCell;

use std::cell::RefCell;

use self::row::Row;
use super::note::{Tag, TagList};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/tag-dialog.ui")]
    pub struct TagDialog {
        #[template_child]
        pub list_view: TemplateChild<gtk::ListView>,

        pub tag_list: OnceCell<TagList>,
        pub other_tag_list: RefCell<TagList>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TagDialog {
        const NAME: &'static str = "NwtyTagDialog";
        type Type = super::TagDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            Row::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TagDialog {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_object(
                        "tag-list",
                        "Tag List",
                        "List of tags",
                        TagList::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpec::new_object(
                        "other-tag-list",
                        "Other Tag List",
                        "The list of tags to compare with",
                        TagList::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "tag-list" => {
                    let tag_list = value.get().unwrap();
                    obj.set_tag_list(tag_list);
                }
                "other-tag-list" => {
                    let other_tag_list = value.get().unwrap();
                    obj.set_other_tag_list(other_tag_list);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "tag-list" => self.tag_list.get().to_value(),
                "other-tag-list" => self.other_tag_list.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(clone!(@weak obj => move |_, list_item| {
                let tag_row = Row::new(&obj.other_tag_list());

                let list_item_expression = gtk::ConstantExpression::new(list_item);
                let item_expression = gtk::PropertyExpression::new(
                    gtk::ListItem::static_type(),
                    Some(&list_item_expression),
                    "item",
                );
                item_expression.bind(&tag_row, "tag", None);

                list_item.set_child(Some(&tag_row));
            }));
            self.list_view.set_factory(Some(&factory));
        }
    }

    impl WidgetImpl for TagDialog {}
    impl WindowImpl for TagDialog {}
    impl AdwWindowImpl for TagDialog {}
}

glib::wrapper! {
    pub struct TagDialog(ObjectSubclass<imp::TagDialog>)
        @extends gtk::Widget, gtk::Window, adw::Window,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl TagDialog {
    pub fn new(tag_list: &TagList, other_tag_list: &TagList) -> Self {
        glib::Object::new(&[("tag-list", tag_list), ("other-tag-list", other_tag_list)])
            .expect("Failed to create TagDialog.")
    }

    fn other_tag_list(&self) -> TagList {
        let imp = imp::TagDialog::from_instance(self);
        imp.other_tag_list.borrow().clone()
    }

    fn set_other_tag_list(&self, other_tag_list: TagList) {
        let imp = imp::TagDialog::from_instance(self);
        imp.other_tag_list.replace(other_tag_list);
    }

    fn set_tag_list(&self, tag_list: TagList) {
        let imp = imp::TagDialog::from_instance(self);

        let selection_model = gtk::NoSelection::new(Some(&tag_list));
        imp.list_view.set_model(Some(&selection_model));
    }
}

mod row;

use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
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
        type ParentType = gtk::Dialog;

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
        }
    }

    impl WidgetImpl for TagDialog {}
    impl WindowImpl for TagDialog {}
    impl DialogImpl for TagDialog {}
}

glib::wrapper! {
    pub struct TagDialog(ObjectSubclass<imp::TagDialog>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl TagDialog {
    pub fn new(tag_list: TagList) -> Self {
        glib::Object::new(&[("tag-list", &tag_list)]).expect("Failed to create TagDialog.")
    }

    pub fn set_other_tag_list(&self, other_tag_list: TagList) {
        let imp = imp::TagDialog::from_instance(self);
        imp.other_tag_list.replace(other_tag_list);
        self.notify("other-tag-list");
    }

    fn set_tag_list(&self, tag_list: TagList) {
        let imp = imp::TagDialog::from_instance(self);

        let selection_model = gtk::SingleSelection::new(Some(&tag_list));
        imp.list_view.set_model(Some(&selection_model));
    }
}

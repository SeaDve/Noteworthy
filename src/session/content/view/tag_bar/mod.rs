mod row;

use adw::{prelude::*, subclass::prelude::*};
use gtk::{glib, subclass::prelude::*, CompositeTemplate};

use self::row::Row;
use crate::model::NoteTagList;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content-view-tag-bar.ui")]
    pub struct TagBar {
        #[template_child]
        pub list_view: TemplateChild<gtk::ListView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TagBar {
        const NAME: &'static str = "NwtyContentViewTagBar";
        type Type = super::TagBar;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Row::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TagBar {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "tag-list",
                    "Tag List",
                    "The model of this view",
                    NoteTagList::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
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
                    obj.set_tag_list(&tag_list);
                }
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for TagBar {}
    impl BinImpl for TagBar {}
}

glib::wrapper! {
    pub struct TagBar(ObjectSubclass<imp::TagBar>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl TagBar {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create TagBar")
    }

    pub fn set_tag_list(&self, tag_list: &NoteTagList) {
        let imp = imp::TagBar::from_instance(self);

        let selection_model = gtk::NoSelection::new(Some(tag_list));
        imp.list_view.set_model(Some(&selection_model));

        self.notify("tag-list");
    }
}

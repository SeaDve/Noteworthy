use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    glib::{self, clone},
    subclass::prelude::*,
    CompositeTemplate,
};

use std::cell::RefCell;

use super::{Tag, TagEditor};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/tag-editor-row.ui")]
    pub struct Row {
        #[template_child]
        pub entry: TemplateChild<gtk::Entry>,

        pub binding: RefCell<Option<glib::Binding>>,

        pub tag: RefCell<Option<Tag>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Row {
        const NAME: &'static str = "NwtyTagEditorRow";
        type Type = super::Row;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("tag-editor-row.done-rename", None, move |obj, _, _| {
                let imp = imp::Row::from_instance(obj);
                let tag_list = obj
                    .root()
                    .unwrap()
                    .downcast::<TagEditor>()
                    .unwrap()
                    .tag_list();
                if let Err(err) = tag_list.rename_tag(&obj.tag().unwrap(), &imp.entry.text()) {
                    log::warn!("Existing tag: {}", err);
                }
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Row {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "tag",
                    "tag",
                    "The tag represented by this row",
                    Tag::static_type(),
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
                "tag" => {
                    let tag = value.get().unwrap();
                    obj.set_tag(tag);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "tag" => obj.tag().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl WidgetImpl for Row {}
    impl BinImpl for Row {}
}

glib::wrapper! {
    pub struct Row(ObjectSubclass<imp::Row>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl Row {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Row")
    }

    fn set_tag(&self, tag: Option<Tag>) {
        let imp = imp::Row::from_instance(self);

        if let Some(binding) = imp.binding.take() {
            binding.unbind();
        }

        if let Some(ref tag) = tag {
            imp.entry.set_text(&tag.name());
            imp.entry
                .connect_text_notify(clone!(@weak tag, @weak self as obj => move |entry| {
                    let tag_list = obj.root().unwrap().downcast::<TagEditor>().unwrap().tag_list();
                    let new_name = entry.text();

                    if tag_list.contains_with_name(&new_name) && new_name != tag.name() {
                        entry.add_css_class("error");
                    } else {
                        entry.remove_css_class("error");
                    }
                }));
        }

        imp.tag.replace(tag);
        self.notify("tag");
    }

    fn tag(&self) -> Option<Tag> {
        let imp = imp::Row::from_instance(self);
        imp.tag.borrow().clone()
    }
}

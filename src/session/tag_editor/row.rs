use gtk::{
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use std::cell::RefCell;

use super::TagEditor;
use crate::model::Tag;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

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
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("tag-editor-row.delete-tag", None, move |obj, _, _| {
                let tag_editor = obj.root().unwrap().downcast::<TagEditor>().unwrap();
                let tag_list = tag_editor.tag_list();
                let note_list = tag_editor.note_list();

                // TODO add confirmation dialog before deleting tag

                let tag = obj.tag().unwrap();

                tag_list.remove(&tag).unwrap();
                note_list.remove_tag_on_all(&tag);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Row {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
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

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for Row {}
}

glib::wrapper! {
    pub struct Row(ObjectSubclass<imp::Row>)
        @extends gtk::Widget;
}

impl Row {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Row")
    }

    fn set_tag(&self, tag: Option<Tag>) {
        let imp = self.imp();

        if let Some(binding) = imp.binding.take() {
            binding.unbind();
        }

        if let Some(ref tag) = tag {
            imp.entry.set_text(&tag.name());
            imp.entry
                .connect_text_notify(clone!(@weak tag, @weak self as obj => move |entry| {
                    let tag_list = obj.root().unwrap().downcast::<TagEditor>().unwrap().tag_list();
                    let tag = obj.tag().unwrap();
                    let new_name = entry.text();

                    if new_name != tag.name() && tag_list.rename_tag(&tag, &new_name).is_err() {
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
        self.imp().tag.borrow().clone()
    }
}

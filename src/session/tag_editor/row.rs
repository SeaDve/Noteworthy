use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    glib::{self, clone},
    subclass::prelude::*,
    CompositeTemplate,
};

use std::cell::RefCell;

use super::TagEditor;
use crate::model::Tag;

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

            klass.install_action("tag-editor-row.delete-tag", None, move |obj, _, _| {
                let tag_editor = obj.root().unwrap().downcast::<TagEditor>().unwrap();
                let tag_list = tag_editor.tag_list();
                let note_list = tag_editor.note_list();

                // TODO add confirmation dialog before deleting tag

                let tag = obj.tag().unwrap();

                tag_list.remove(&tag).unwrap();

                // FIXME make this faster
                for note in note_list.iter() {
                    let metadata = note.metadata();
                    let note_tag_list = metadata.tag_list();
                    if let Err(err) = note_tag_list.remove(&tag) {
                        log::warn!(
                            "Failed to remove tag {} on note {}: {}",
                            tag.name(),
                            metadata.title(),
                            err
                        );
                    }
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
        let imp = imp::Row::from_instance(self);
        imp.tag.borrow().clone()
    }
}

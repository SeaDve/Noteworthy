use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/note_view.ui")]
    pub struct NoteView {
        #[template_child]
        pub label: TemplateChild<gtk::Label>,

        pub content: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NoteView {
        const NAME: &'static str = "NwtyNoteView";
        type Type = super::NoteView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NoteView {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_string(
                    "content",
                    "Content",
                    "Content of the view",
                    None,
                    glib::ParamFlags::READWRITE,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "content" => {
                    let content = value.get().unwrap();
                    self.content.replace(content);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "content" => self.content.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for NoteView {}
    impl BinImpl for NoteView {}
}

glib::wrapper! {
    pub struct NoteView(ObjectSubclass<imp::NoteView>)
        @extends gtk::Widget, adw::Bin;
}

impl NoteView {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create NoteView.")
    }

    pub fn set_content(&self, content: &str) {
        self.set_property("content", content).unwrap();
    }
}

use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

use std::cell::RefCell;

use super::Note;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content_view.ui")]
    pub struct ContentView {
        #[template_child]
        pub label: TemplateChild<gtk::Label>,

        pub note: RefCell<Option<Note>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContentView {
        const NAME: &'static str = "NwtyContentView";
        type Type = super::ContentView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ContentView {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "note",
                    "Note",
                    "Current note in the view",
                    Note::static_type(),
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
                "note" => {
                    let note: Option<Note> = value.get().unwrap();

                    if let Some(ref note) = note {
                        self.label.set_label(&note.content());
                    }

                    self.note.replace(note);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "note" => self.note.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ContentView {}
    impl BinImpl for ContentView {}
}

glib::wrapper! {
    pub struct ContentView(ObjectSubclass<imp::ContentView>)
        @extends gtk::Widget, adw::Bin;
}

impl ContentView {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ContentView.")
    }

    pub fn set_note(&self, note: Option<&Note>) {
        self.set_property("note", note).unwrap();
    }
}

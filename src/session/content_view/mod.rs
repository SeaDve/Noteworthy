use adw::subclass::prelude::*;
use gtk::{
    gdk,
    glib::{self, clone, signal::Inhibit},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};
use sourceview::prelude::*;

use std::cell::RefCell;

use super::note::{Note, NoteExt};
use crate::{error::Error, Result};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content_view.ui")]
    pub struct ContentView {
        #[template_child]
        pub view: TemplateChild<sourceview::View>,

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

            let key_events = gtk::EventControllerKey::new();
            key_events
                .connect_key_pressed(clone!(@weak obj => @default-return Inhibit(false), move |_, key, _, modifier| {
                    if modifier.contains(gdk::ModifierType::CONTROL_MASK) && key == gdk::keys::constants::s {
                        obj.save_active_note().unwrap();
                        log::info!("File saved");
                        Inhibit(true)
                    } else {
                        Inhibit(false)
                    }
                }));
            self.view.add_controller(&key_events);
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
                        let buffer: sourceview::Buffer = self.view.buffer().downcast().unwrap();

                        let md_lang = sourceview::LanguageManager::default()
                            .and_then(|lm| lm.language("markdown"));
                        buffer.set_language(md_lang.as_ref());

                        buffer.set_text(&note.content());

                        self.view.grab_focus();
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

    pub fn note(&self) -> Option<Note> {
        self.property("note").unwrap().get().unwrap()
    }

    pub fn set_note(&self, note: Option<&Note>) {
        self.set_property("note", note).unwrap();
    }

    pub fn save_active_note(&self) -> Result<()> {
        let imp = imp::ContentView::from_instance(self);

        let note = self.note().ok_or_else(|| {
            Error::Note("Cannot save active note, the view doesn't containt a note".to_string())
        })?;

        let buffer: sourceview::Buffer = imp.view.buffer().downcast().unwrap();
        let (start_iter, end_iter) = buffer.bounds();

        note.set_content(&buffer.text(&start_iter, &end_iter, true))?;

        Ok(())
    }
}

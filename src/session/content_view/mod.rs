mod content_header;

use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use sourceview::prelude::*;

use std::cell::{Cell, RefCell};

use self::content_header::ContentHeader;
use super::manager::Note;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/content_view.ui")]
    pub struct ContentView {
        #[template_child]
        pub source_view: TemplateChild<sourceview::View>,
        #[template_child]
        pub content_header: TemplateChild<ContentHeader>,

        pub title_binding: RefCell<Option<glib::Binding>>,

        pub compact: Cell<bool>,
        pub note: RefCell<Option<Note>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContentView {
        const NAME: &'static str = "NwtyContentView";
        type Type = super::ContentView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            ContentHeader::static_type();
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
                vec![
                    glib::ParamSpec::new_boolean(
                        "compact",
                        "Compact",
                        "Whether it is compact view mode",
                        false,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_object(
                        "note",
                        "Note",
                        "Current note in the view",
                        Note::static_type(),
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
                "compact" => {
                    let compact = value.get().unwrap();
                    self.compact.set(compact);
                }
                "note" => {
                    let note = value.get().unwrap();
                    obj.set_note(note);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "compact" => self.compact.get().to_value(),
                "note" => obj.note().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ContentView {}
    impl BoxImpl for ContentView {}
}

glib::wrapper! {
    pub struct ContentView(ObjectSubclass<imp::ContentView>)
        @extends gtk::Widget, gtk::Box;
}

impl ContentView {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ContentView.")
    }

    pub fn note(&self) -> Option<Note> {
        let imp = imp::ContentView::from_instance(self);
        imp.note.borrow().clone()
    }

    pub fn set_note(&self, note: Option<Note>) {
        let imp = imp::ContentView::from_instance(self);
        // this unbinds before binding it later
        if let Some(title_binding) = imp.title_binding.take() {
            title_binding.unbind();
        }

        if let Some(ref note) = note {
            imp.source_view.grab_focus();
            let buffer: sourceview::Buffer = imp.source_view.buffer().downcast().unwrap();

            let md_lang =
                sourceview::LanguageManager::default().and_then(|lm| lm.language("markdown"));
            buffer.set_language(md_lang.as_ref());
            buffer.set_text(&note.content());

            // FIXME make this not hacky
            let title_binding = note
                .bind_property("title", &imp.content_header.get(), "title")
                .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::BIDIRECTIONAL)
                .build();

            imp.title_binding.replace(title_binding);
        }

        imp.note.replace(note);
        self.notify("note");
    }

    pub fn save_active_note(&self) {
        // TODO maybe there is better place to put this functionality

        match self.note() {
            Some(note) => {
                let imp = imp::ContentView::from_instance(self);
                let buffer: sourceview::Buffer = imp.source_view.buffer().downcast().unwrap();
                let (start_iter, end_iter) = buffer.bounds();
                let buffer_text = buffer.text(&start_iter, &end_iter, true);

                note.set_content(&buffer_text);
            }
            None => log::warn!("No note found on the view, not saving the content"),
        };
    }
}

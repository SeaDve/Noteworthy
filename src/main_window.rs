use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use std::path::Path;

use crate::{
    application::Application,
    config::PROFILE,
    session::{
        note::{Note, NoteExt},
        provider::{LocalProvider, Provider},
        ContentView, Sidebar,
    },
};

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/main_window.ui")]
    pub struct MainWindow {
        #[template_child]
        pub notes_sidebar: TemplateChild<Sidebar>,
        #[template_child]
        pub note_view: TemplateChild<ContentView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainWindow {
        const NAME: &'static str = "NwtyMainWindow";
        type Type = super::MainWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();

            Sidebar::static_type();
            ContentView::static_type();
        }
    }

    impl ObjectImpl for MainWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            obj.load_window_size();

            let note_provider = LocalProvider::new(Path::new("/home/dave/Notes"));
            let notes_list = note_provider.retrive_notes().unwrap();

            self.notes_sidebar
                .set_model(Some(&gtk::SingleSelection::new(Some(&notes_list))));

            self.notes_sidebar
                .connect_activate(clone!(@weak obj => move |notes_sidebar, pos| {
                    let selected_note: Note = notes_sidebar
                        .model()
                        .unwrap()
                        .item(pos)
                        .unwrap()
                        .downcast()
                        .unwrap();

                    dbg!(selected_note.title());

                    let imp = obj.private();
                    imp.note_view.set_note(Some(&selected_note));
                }));
        }
    }

    impl WidgetImpl for MainWindow {}
    impl WindowImpl for MainWindow {
        fn close_request(&self, window: &Self::Type) -> gtk::Inhibit {
            if let Err(err) = window.save_window_size() {
                log::warn!("Failed to save window state, {}", &err);
            }

            self.parent_close_request(window)
        }
    }

    impl ApplicationWindowImpl for MainWindow {}
    impl AdwApplicationWindowImpl for MainWindow {}
}

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create MainWindow.")
    }

    fn private(&self) -> &imp::MainWindow {
        imp::MainWindow::from_instance(self)
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let settings = Application::default().settings();

        let (width, height) = self.default_size();

        settings.set_int("window-width", width)?;
        settings.set_int("window-height", height)?;

        settings.set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let settings = Application::default().settings();

        let width = settings.int("window-width");
        let height = settings.int("window-height");
        let is_maximized = settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }
}

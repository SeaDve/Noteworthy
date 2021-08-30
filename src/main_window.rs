use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};

use crate::{
    application::Application,
    config::{APP_ID, PROFILE},
    model::{Note, NotesList},
    widgets::NotesSidebar,
};

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/main_window.ui")]
    pub struct MainWindow {
        #[template_child]
        pub headerbar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub notes_sidebar: TemplateChild<NotesSidebar>,

        pub settings: gio::Settings,
    }

    impl Default for MainWindow {
        fn default() -> Self {
            Self {
                headerbar: TemplateChild::default(),
                notes_sidebar: TemplateChild::default(),

                settings: gio::Settings::new(APP_ID),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainWindow {
        const NAME: &'static str = "NoteworthyMainWindow";
        type Type = super::MainWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();

            NotesSidebar::static_type();
        }
    }

    impl ObjectImpl for MainWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            obj.load_window_size();

            let note1 = Note::new();
            note1.set_title("A note");
            note1.set_content("This note contains a note");

            let note2 = Note::new();
            note2.set_title("Another note");
            note2.set_content("This note contains another note");

            let notes_list = NotesList::new();
            notes_list.add_note(note1);
            notes_list.add_note(note2);
            self.notes_sidebar
                .set_model(Some(&gtk::SingleSelection::new(Some(&notes_list))));
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

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let imp = imp::MainWindow::from_instance(self);

        let (width, height) = self.default_size();

        imp.settings.set_int("window-width", width)?;
        imp.settings.set_int("window-height", height)?;

        imp.settings
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let imp = imp::MainWindow::from_instance(self);

        let width = imp.settings.int("window-width");
        let height = imp.settings.int("window-height");
        let is_maximized = imp.settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }
}
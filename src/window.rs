use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};
use once_cell::unsync::OnceCell;

use crate::{application::Application, config::PROFILE, session::Session, setup::Setup, utils};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/window.ui")]
    pub struct Window {
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub setup: TemplateChild<Setup>,

        pub session: OnceCell<Session>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "NwtyWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Setup::static_type();
            Self::bind_template(klass);

            klass.install_action("win.close", None, move |obj, _, _| {
                obj.close();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            self.setup
                .connect_session_setup_done(clone!(@weak obj => move |_| {
                    obj.load_session();
                    obj.switch_to_session_page();
                }));

            if utils::default_notes_dir().exists() {
                obj.load_session();
                obj.switch_to_session_page();
            }

            obj.load_window_size();
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        fn close_request(&self, obj: &Self::Type) -> gtk::Inhibit {
            if let Err(err) = obj.save_window_size() {
                log::warn!("Failed to save window state, {}", &err);
            }

            // TODO what if app crashed?
            if let Some(session) = self.session.get() {
                session.save();
            }

            self.parent_close_request(obj)
        }
    }

    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create Window.")
    }

    pub fn session(&self) -> Session {
        let imp = imp::Window::from_instance(self);
        imp.session.get().expect("Call load_session first").clone()
    }

    fn load_session(&self) {
        let imp = imp::Window::from_instance(self);

        let notes_folder = gio::File::for_path(&utils::default_notes_dir());
        let session = Session::new(&notes_folder);
        imp.main_stack.add_child(&session);
        imp.session.set(session).unwrap();
    }

    fn switch_to_session_page(&self) {
        let imp = imp::Window::from_instance(self);
        imp.main_stack.set_visible_child(&self.session());
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

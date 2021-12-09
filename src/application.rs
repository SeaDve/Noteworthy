use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};

use crate::{
    config::{APP_ID, PKGDATADIR, PROFILE, VERSION},
    window::Window,
};

mod imp {
    use super::*;
    use glib::WeakRef;
    use once_cell::unsync::OnceCell;

    #[derive(Debug)]
    pub struct Application {
        pub window: OnceCell<WeakRef<Window>>,
        pub settings: gio::Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "NwtyApplication";
        type Type = super::Application;
        type ParentType = adw::Application;

        fn new() -> Self {
            Self {
                window: OnceCell::new(),
                settings: gio::Settings::new(APP_ID),
            }
        }
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self, obj: &Self::Type) {
            if let Some(window) = self.window.get() {
                let window = window.upgrade().unwrap();
                window.show();
                window.present();
                return;
            }

            let window = Window::new(obj);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            obj.main_window().present();
        }

        fn startup(&self, obj: &Self::Type) {
            self.parent_startup(obj);

            gtk::Window::set_default_icon_name(APP_ID);

            obj.setup_gactions();
            obj.setup_accels();
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &Some(APP_ID)),
            ("flags", &gio::ApplicationFlags::empty()),
            ("resource-base-path", &Some("/io/github/seadve/Noteworthy/")),
        ])
        .expect("Application initialization failed...")
    }

    pub fn run(&self) {
        log::info!("Noteworthy ({})", APP_ID);
        log::info!("Version: {} ({})", VERSION, PROFILE);
        log::info!("Datadir: {}", PKGDATADIR);

        ApplicationExtManual::run(self);
    }

    pub fn settings(&self) -> gio::Settings {
        let imp = imp::Application::from_instance(self);
        imp.settings.clone()
    }

    pub fn main_window(&self) -> Window {
        let imp = imp::Application::from_instance(self);
        imp.window.get().unwrap().upgrade().unwrap()
    }

    fn show_about_dialog(&self) {
        let dialog = gtk::AboutDialogBuilder::new()
            .transient_for(&self.main_window())
            .modal(true)
            // .comments(&gettext("Elegantly record your screen"))
            .version(VERSION)
            .logo_icon_name(APP_ID)
            .authors(vec!["Dave Patrick".into()])
            // Translators: Replace "translator-credits" with your names. Put a comma between.
            .translator_credits(&gettext("translator-credits"))
            .copyright(&gettext("Copyright 2021 Dave Patrick"))
            .license_type(gtk::License::Gpl30)
            .website("https://github.com/SeaDve/Noteworthy")
            .website_label(&gettext("GitHub"))
            .build();

        dialog.show();
    }

    fn setup_gactions(&self) {
        let action_quit = gio::SimpleAction::new("quit", None);
        action_quit.connect_activate(clone!(@weak self as obj => move |_, _| {
            // This is needed to trigger the delete event and saving the window state
            obj.main_window().close();
            obj.quit();
        }));
        self.add_action(&action_quit);

        let action_about = gio::SimpleAction::new("about", None);
        action_about.connect_activate(clone!(@weak self as obj => move |_, _| {
            obj.show_about_dialog();
        }));
        self.add_action(&action_about);
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<primary>q"]);
    }
}

impl Default for Application {
    fn default() -> Self {
        gio::Application::default().unwrap().downcast().unwrap()
    }
}

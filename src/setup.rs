use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, subclass::Signal},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

use super::session::Session;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/setup.ui")]
    pub struct Setup {}

    #[glib::object_subclass]
    impl ObjectSubclass for Setup {
        const NAME: &'static str = "NwtySetup";
        type Type = super::Setup;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("setup.create-session", None, move |obj, _, _| {
                let notes_folder = {
                    let mut data_dir = glib::user_data_dir();
                    data_dir.push("Notes");
                    gio::File::for_path(&data_dir)
                };

                // FIXME make this async
                // TODO Add separate load existing session so not always query_exists
                // TODO maybe move this to window
                if !notes_folder.query_exists(None::<&gio::Cancellable>) {
                    log::info!("Note folder not found, creating...");
                    if let Err(err) = notes_folder.make_directory(None::<&gio::Cancellable>) {
                        log::error!("Failed to create note folder, {:?}", err);
                    }
                }

                let session = Session::new(&notes_folder);
                obj.emit_by_name("new-session", &[&session]).unwrap();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Setup {
        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    "new-session",
                    &[Session::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for Setup {}
    impl BoxImpl for Setup {}
}

glib::wrapper! {
    pub struct Setup(ObjectSubclass<imp::Setup>)
        @extends gtk::Widget, gtk::Box;
}

impl Setup {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Setup.")
    }

    pub fn connect_new_session<F: Fn(&Self, &Session) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("new-session", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let session = values[1].get::<Session>().unwrap();
            f(&obj, &session);
            None
        })
        .unwrap()
    }
}

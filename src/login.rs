use adw::subclass::prelude::*;
use gtk::{
    glib::{self, subclass::Signal},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

use super::{components::FileChooserButton, session::Session};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/login.ui")]
    pub struct Login {
        #[template_child]
        pub file_chooser_button: TemplateChild<FileChooserButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Login {
        const NAME: &'static str = "NwtyLogin";
        type Type = super::Login;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            FileChooserButton::static_type();
            Self::bind_template(klass);

            klass.install_action("login.new-session", None, move |obj, _, _| {
                let imp = imp::Login::from_instance(obj);

                if let Some(folder) = imp.file_chooser_button.current_folder() {
                    let session = Session::new(&folder);
                    obj.emit_by_name("new-session", &[&session]).unwrap();
                } else {
                    // FIXME handle this
                    log::warn!("File chooser button has no file!");
                }
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Login {
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

    impl WidgetImpl for Login {}
    impl BoxImpl for Login {}
}

glib::wrapper! {
    pub struct Login(ObjectSubclass<imp::Login>)
        @extends gtk::Widget, gtk::Box;
}

impl Login {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Login.")
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

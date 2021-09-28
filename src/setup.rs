use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone, subclass::Signal},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

use crate::utils;

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

            klass.install_action("setup.setup-session", None, move |obj, _, _| {
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak obj => async move {
                    obj.setup_session().await;
                    obj.emit_by_name("session-setup-done", &[]).unwrap();
                }));
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
                vec![Signal::builder("session-setup-done", &[], <()>::static_type().into()).build()]
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

    pub fn connect_session_setup_done<F: Fn(&Self) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("session-setup-done", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);
            None
        })
        .unwrap()
    }

    async fn setup_session(&self) {
        let notes_folder = gio::File::for_path(&utils::default_notes_dir());
        if let Err(err) = notes_folder
            .make_directory_async_future(glib::PRIORITY_HIGH_IDLE)
            .await
        {
            // TODO add user facing error dialog
            log::error!("Failed to create note folder, {:?}", err);
        }
    }
}

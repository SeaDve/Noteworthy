use adw::subclass::prelude::*;
use gtk::{
    gio,
    glib::{self, clone, subclass::Signal},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

use crate::{repository::Repository, utils};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/setup.ui")]
    pub struct Setup {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub welcome: TemplateChild<gtk::Box>,
        #[template_child]
        pub select_provider: TemplateChild<gtk::Box>,
        #[template_child]
        pub repo_url_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub passphrase_entry: TemplateChild<gtk::Entry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Setup {
        const NAME: &'static str = "NwtySetup";
        type Type = super::Setup;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            // TODO consider changing these action names
            klass.install_action("setup.setup-session", None, move |obj, _, _| {
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak obj => async move {
                    obj.setup_session().await;
                    obj.emit_by_name("session-setup-done", &[]).unwrap();
                }));
            });

            klass.install_action("setup.go-back-welcome", None, move |obj, _, _| {
                let imp = imp::Setup::from_instance(obj);
                imp.stack.set_visible_child(&imp.welcome.get());
            });

            klass.install_action("setup.setup-git-host", None, move |obj, _, _| {
                let imp = imp::Setup::from_instance(obj);
                imp.stack.set_visible_child(&imp.select_provider.get());
            });

            klass.install_action("setup.enter-repo-url", None, move |obj, _, _| {
                let imp = imp::Setup::from_instance(obj);
                let repo_url = imp.repo_url_entry.text();
                let passphrase = imp.passphrase_entry.text();

                let repo = Repository::new(repo_url.to_string(), glib::user_data_dir());
                if let Err(err) = repo.clone(Some(&passphrase)) {
                    log::error!("Failed to clone: {}", err);
                }
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
    impl BinImpl for Setup {}
}

glib::wrapper! {
    pub struct Setup(ObjectSubclass<imp::Setup>)
        @extends gtk::Widget, adw::Bin;
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

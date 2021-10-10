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
    pub struct Setup {
        #[template_child]
        pub navigate_back_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub navigate_forward_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub content: TemplateChild<adw::Leaflet>,
        #[template_child]
        pub welcome: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub select_provider: TemplateChild<adw::StatusPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Setup {
        const NAME: &'static str = "NwtySetup";
        type Type = super::Setup;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("setup.navigate-back", None, move |obj, _, _| {
                let imp = imp::Setup::from_instance(obj);
                imp.content.navigate(adw::NavigationDirection::Back);
            });

            klass.install_action("setup.navigate-forward", None, move |obj, _, _| {
                let imp = imp::Setup::from_instance(obj);
                imp.content.navigate(adw::NavigationDirection::Forward);
            });

            // TODO consider changing these action names
            klass.install_action("setup.setup-session", None, move |obj, _, _| {
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak obj => async move {
                    obj.setup_session().await;
                    obj.emit_by_name("session-setup-done", &[]).unwrap();
                }));
            });

            klass.install_action("setup.setup-git-host", None, move |obj, _, _| {
                let imp = imp::Setup::from_instance(obj);
                imp.content.set_visible_child(&imp.select_provider.get());
            });

            // klass.install_action("setup.enter-repo-url", None, move |obj, _, _| {
            //     let imp = imp::Setup::from_instance(obj);
            //     let repo_url = imp.repo_url_entry.text();
            //     let passphrase = imp.passphrase_entry.text();

            //     let ctx = glib::MainContext::default();
            //     ctx.spawn_local(async move {
            //         let repo_path = gio::File::for_path(&glib::user_data_dir());
            //         let repo = Repository::new(&repo_path);
            //         if let Err(err) = repo
            //             .clone(repo_url.to_string(), passphrase.to_string())
            //             .await
            //         {
            //             log::error!("Failed to clone: {}", err);
            //         } else {
            //             log::info!("Successfull repo clone");
            //         }
            //     });
            // });
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

        fn constructed(&self, obj: &Self::Type) {
            self.content
                .connect_visible_child_notify(clone!(@weak obj => move |content| {
                    let imp = imp::Setup::from_instance(&obj);
                    let is_main_page = content.visible_child() == Some(imp.welcome.get().upcast());
                    imp.navigate_back_button.set_visible(!is_main_page);
                    imp.navigate_forward_button.set_visible(!is_main_page);
                }));
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

use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    gio,
    glib::{self, clone, subclass::Signal},
    subclass::prelude::*,
    CompositeTemplate,
};

use std::cell::RefCell;

use crate::{core::NoteRepository, utils};

mod imp {
    use super::*;

    #[repr(u8)]
    #[derive(Debug, PartialEq)]
    pub enum GitHost {
        Github = 0,
        Gitlab = 1,
        Custom = 2,
    }

    impl GitHost {
        pub fn from_int(int: u8) -> Self {
            unsafe { std::mem::transmute(int) }
        }
    }

    #[derive(Debug, Default)]
    pub struct SetupConfig {
        pub provider: Option<GitHost>,
        pub is_automatic: Option<bool>,
        pub clone_url: Option<String>,
    }

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/setup.ui")]
    pub struct Setup {
        #[template_child]
        pub navigate_back_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub navigate_forward_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub content: TemplateChild<adw::Leaflet>,

        // select provider page
        #[template_child]
        pub git_host_provider_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub is_automatic_switch: TemplateChild<gtk::Switch>,

        // create repo page
        #[template_child]
        pub clone_url_entry: TemplateChild<gtk::Entry>,

        pub config: RefCell<SetupConfig>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Setup {
        const NAME: &'static str = "NwtySetup";
        type Type = super::Setup;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("setup.navigate-back", None, move |obj, _, _| {
                obj.navigate_back();
            });

            klass.install_action("setup.navigate-forward", None, move |obj, _, _| {
                obj.navigate_forward();
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
                imp.content.set_visible_child_name("select-provider");
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
                    let is_main_page = content.visible_child_name().unwrap().as_str() == "welcome";
                    imp.navigate_back_button.set_visible(!is_main_page);
                    imp.navigate_forward_button.set_visible(!is_main_page);
                }));

            self.clone_url_entry
                .connect_text_notify(clone!(@weak obj => move |entry| {
                    let imp = imp::Setup::from_instance(&obj);
                    if imp.content.visible_child_name().unwrap().as_str() == "create-repo" {
                        let entry_text = entry.text();
                        let is_valid = NoteRepository::validate_remote_url(&entry_text);
                        obj.action_set_enabled("setup.navigate-forward", is_valid);
                    }
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

    fn navigate_forward(&self) {
        let imp = imp::Setup::from_instance(self);
        let visible_page_name = imp.content.visible_child_name().unwrap();

        match visible_page_name.as_str() {
            "select-provider" => {
                self.select_provider();

                imp.content.set_visible_child_name("create-repo");
                imp.clone_url_entry.notify("text");
            }
            "create-repo" => {
                self.create_repo();
            }
            other => unreachable!("Invalid page name '{}'", other),
        }
    }

    fn navigate_back(&self) {
        let imp = imp::Setup::from_instance(self);
        let visible_page_name = imp.content.visible_child_name().unwrap();

        match visible_page_name.as_str() {
            "select-provider" => {
                imp.content.set_visible_child_name("welcome");
            }
            "create-repo" => {
                imp.content.set_visible_child_name("select-provider");
            }
            other => unreachable!("Invalid page name '{}'", other),
        }

        self.action_set_enabled("setup.navigate-forward", true);
    }

    fn select_provider(&self) {
        let imp = imp::Setup::from_instance(self);
        let mut config = imp.config.borrow_mut();

        dbg!(&config);

        let is_automatic = imp.is_automatic_switch.state();
        config.is_automatic = Some(is_automatic);

        let provider = imp::GitHost::from_int(imp.git_host_provider_row.selected() as u8);
        config.provider = Some(provider);

        dbg!(&config);
    }

    fn create_repo(&self) {
        let imp = imp::Setup::from_instance(self);
        let mut config = imp.config.borrow_mut();

        let clone_url = imp.clone_url_entry.text();
        config.clone_url = Some(clone_url.to_string());

        dbg!(NoteRepository::validate_remote_url(&clone_url));

        dbg!(&config);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn git_host() {
        use imp::GitHost;

        assert_eq!(GitHost::Github, GitHost::from_int(0));
        assert_eq!(GitHost::Gitlab, GitHost::from_int(1));
        assert_eq!(GitHost::Custom, GitHost::from_int(2));
    }

    #[test]
    #[should_panic]
    fn git_host_not_found() {
        use imp::GitHost;

        GitHost::from_int(3);
        GitHost::from_int(u8::MAX);
    }
}

use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*};

use std::cell::Cell;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sync-button.ui")]
    pub struct SyncButton {
        #[template_child]
        pub inner_button: TemplateChild<gtk::Button>,

        pub is_spinning: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SyncButton {
        const NAME: &'static str = "NwtySyncButton";
        type Type = super::SyncButton;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SyncButton {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_string(
                        "action-name",
                        "Action Name",
                        "The action to be called when clicked",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpec::new_boolean(
                        "is-spinning",
                        "Is Spinning",
                        "The action to be called when clicked",
                        false,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "action-name" => {
                    let action_name = value.get().unwrap();
                    self.inner_button.set_action_name(action_name);
                }
                "is-spinning" => {
                    let is_spinning = value.get().unwrap();
                    obj.set_is_spinning(is_spinning);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "action-name" => self.inner_button.action_name().to_value(),
                "is-spinning" => self.is_spinning.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for SyncButton {}
    impl BinImpl for SyncButton {}
}

glib::wrapper! {
    pub struct SyncButton(ObjectSubclass<imp::SyncButton>)
        @extends gtk::Widget, adw::Bin, @implements gtk::Accessible;
}

impl SyncButton {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create SyncButton.")
    }

    pub fn set_is_spinning(&self, is_spinning: bool) {
        let imp = imp::SyncButton::from_instance(self);

        if is_spinning {
            imp.inner_button.add_css_class("spinning");
        } else {
            imp.inner_button.remove_css_class("spinning");
        }

        imp.is_spinning.set(is_spinning);
        self.notify("is-spinning");
    }
}

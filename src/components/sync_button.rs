// TODO remove this file, since it is not currently used

use adw::subclass::prelude::*;
use gtk::{
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sync-button.ui")]
    pub struct SyncButton {
        #[template_child]
        pub inner_button: TemplateChild<gtk::Button>,
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
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_string(
                    "action-name",
                    "Action Name",
                    "The action to be called when clicked",
                    None,
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "action-name" => {
                    let action_name = value.get().unwrap();
                    self.inner_button.set_action_name(action_name);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "action-name" => self.inner_button.action_name().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
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
}

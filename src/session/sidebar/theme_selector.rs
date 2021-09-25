use adw::subclass::prelude::*;
use gtk::{
    glib::{self, GEnum},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate,
};

use std::cell::Cell;

#[derive(Debug, Clone, Copy, GEnum)]
#[genum(type_name = "NwtyTheme")]
pub enum Theme {
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Light
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/sidebar-theme-selector.ui")]
    pub struct ThemeSelector {
        pub theme: Cell<Theme>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ThemeSelector {
        const NAME: &'static str = "NwtySidebarThemeSelector";
        type Type = super::ThemeSelector;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ThemeSelector {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_enum(
                    "theme",
                    "Theme",
                    "Current theme",
                    Theme::static_type(),
                    Theme::Light as i32,
                    glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
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
                "theme" => {
                    let theme = value.get().unwrap();
                    obj.set_theme(theme);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "theme" => obj.theme().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ThemeSelector {}
    impl BinImpl for ThemeSelector {}
}

glib::wrapper! {
    pub struct ThemeSelector(ObjectSubclass<imp::ThemeSelector>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl ThemeSelector {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ThemeSelector.")
    }

    pub fn theme(&self) -> Theme {
        let imp = imp::ThemeSelector::from_instance(self);
        imp.theme.get()
    }

    pub fn set_theme(&self, theme: Theme) {
        let imp = imp::ThemeSelector::from_instance(self);
        imp.theme.set(theme);
        self.notify("theme");
    }
}

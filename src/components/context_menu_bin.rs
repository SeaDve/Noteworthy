// Class from Fractal-next
// See https://gitlab.gnome.org/GNOME/fractal/-/blob/fractal-next/src/components/context_menu_bin.rs

use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, glib::clone, CompositeTemplate};

mod imp {
    use super::*;
    use glib::subclass::InitializingObject;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/context_menu_bin.ui")]
    pub struct ContextMenuBin {
        #[template_child]
        pub click_gesture: TemplateChild<gtk::GestureClick>,
        #[template_child]
        pub long_press_gesture: TemplateChild<gtk::GestureLongPress>,
        pub popover: gtk::PopoverMenu,
    }

    impl Default for ContextMenuBin {
        fn default() -> Self {
            Self {
                click_gesture: Default::default(),
                long_press_gesture: Default::default(),
                // WORKAROUND: there is some issue with creating the popover from the template
                popover: gtk::PopoverMenuBuilder::new()
                    .position(gtk::PositionType::Bottom)
                    .has_arrow(false)
                    .halign(gtk::Align::Start)
                    .build(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContextMenuBin {
        const NAME: &'static str = "NwtyContextMenuBin";
        type Type = super::ContextMenuBin;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("context-menu.activate", None, move |widget, _, _| {
                widget.open_menu_at(0, 0);
            });

            klass.add_binding_action(
                gdk::keys::constants::F10,
                gdk::ModifierType::SHIFT_MASK,
                "context-menu.activate",
                None,
            );

            klass.add_binding_action(
                gdk::keys::constants::Menu,
                gdk::ModifierType::empty(),
                "context-menu.activate",
                None,
            );
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ContextMenuBin {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "context-menu",
                    "Context Menu",
                    "The context menu",
                    gio::MenuModel::static_type(),
                    glib::ParamFlags::READWRITE,
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
                "context-menu" => {
                    let context_menu = value
                        .get::<Option<gio::MenuModel>>()
                        .expect("type conformity checked by `Object::set_property`");
                    obj.set_context_menu(context_menu);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "context-menu" => obj.context_menu().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.popover.set_parent(obj);
            self.long_press_gesture
                .connect_pressed(clone!(@weak obj => move |gesture, x, y| {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    gesture.reset();
                    obj.open_menu_at(x as i32, y as i32);
                }));

            self.click_gesture.connect_released(
                clone!(@weak obj => move |gesture, n_press, x, y| {
                    if n_press > 1 {
                        return;
                    }

                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    obj.open_menu_at(x as i32, y as i32);
                }),
            );
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.popover.unparent();
        }
    }

    impl WidgetImpl for ContextMenuBin {}
    impl BinImpl for ContextMenuBin {}
}

glib::wrapper! {
    /// A Bin widget that adds a context menu.
    pub struct ContextMenuBin(ObjectSubclass<imp::ContextMenuBin>)
        @extends gtk::Widget, adw::Bin, @implements gtk::Accessible;
}

impl ContextMenuBin {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ContextMenuBin.")
    }

    fn open_menu_at(&self, x: i32, y: i32) {
        let priv_ = imp::ContextMenuBin::from_instance(self);
        let popover = &priv_.popover;

        log::debug!("Context menu was activated");

        if popover.menu_model().is_none() {
            return;
        }

        popover.set_pointing_to(&gdk::Rectangle {
            x,
            y,
            width: 0,
            height: 0,
        });
        popover.popup();
    }
}

pub trait ContextMenuBinExt: 'static {
    /// Set the `MenuModel` used in the context menu.
    fn set_context_menu(&self, menu: Option<gio::MenuModel>);

    /// Get the `MenuModel` used in the context menu.
    fn context_menu(&self) -> Option<gio::MenuModel>;
}

impl<O: IsA<ContextMenuBin>> ContextMenuBinExt for O {
    fn set_context_menu(&self, menu: Option<gio::MenuModel>) {
        let priv_ = imp::ContextMenuBin::from_instance(self.upcast_ref());
        priv_.popover.set_menu_model(menu.as_ref());
    }

    fn context_menu(&self) -> Option<gio::MenuModel> {
        let priv_ = imp::ContextMenuBin::from_instance(self.upcast_ref());
        priv_.popover.menu_model()
    }
}

pub trait ContextMenuBinImpl: BinImpl {}

unsafe impl<T: ContextMenuBinImpl> IsSubclassable<T> for ContextMenuBin {
    fn class_init(class: &mut glib::Class<Self>) {
        <gtk::Widget as IsSubclassable<T>>::class_init(class);
    }
    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <gtk::Widget as IsSubclassable<T>>::instance_init(instance);
    }
}

impl Default for ContextMenuBin {
    fn default() -> Self {
        Self::new()
    }
}

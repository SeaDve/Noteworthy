use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk::{
    gdk, gio,
    glib::{self, clone},
    subclass::prelude::*,
};

use std::cell::{Cell, RefCell};

use super::ScrollablePicture;
use crate::{core::FileType, model::Attachment, spawn, spawn_blocking};

mod imp {
    use super::*;
    use glib::{subclass::Signal, WeakRef};
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/picture-viewer.ui")]
    pub struct PictureViewer {
        #[template_child]
        pub flap: TemplateChild<adw::Flap>,
        #[template_child]
        pub picture: TemplateChild<ScrollablePicture>,
        #[template_child]
        pub fullscreen_button: TemplateChild<gtk::Button>,

        pub attachment: RefCell<Option<WeakRef<Attachment>>>,
        pub fullscreened: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PictureViewer {
        const NAME: &'static str = "NwtyPictureViewer";
        type Type = super::PictureViewer;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);

            klass.install_action("picture-viewer.exit", None, move |obj, _, _| {
                obj.on_exit();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PictureViewer {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("on-exit", &[], <()>::static_type().into()).build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "attachment",
                        "Attachment",
                        "Attachment shown by PictureViewer",
                        Attachment::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecBoolean::new(
                        "fullscreened",
                        "Fullscreened",
                        "Whether the viewer is fullscreened",
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
                "attachment" => {
                    let attachment = value.get().unwrap();
                    obj.set_attachment(attachment);
                }
                "fullscreened" => {
                    let fullscreened = value.get().unwrap();
                    obj.set_fullscreened(fullscreened);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "attachment" => obj.attachment().to_value(),
                "fullscreened" => obj.is_fullscreened().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.update_ui();
        }
    }

    impl WidgetImpl for PictureViewer {}
    impl BinImpl for PictureViewer {}
}

glib::wrapper! {
    pub struct PictureViewer(ObjectSubclass<imp::PictureViewer>)
        @extends gtk::Widget, adw::Bin;
}

#[gtk::template_callbacks]
impl PictureViewer {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create PictureViewer.")
    }

    pub fn connect_on_exit<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_local("on-exit", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);
            None
        })
    }

    pub fn set_attachment(&self, attachment: Option<Attachment>) {
        if attachment == self.attachment() {
            return;
        }

        self.imp()
            .attachment
            .replace(attachment.map(|attachment| attachment.downgrade()));
        self.update_content();
        self.notify("attachment");
    }

    pub fn attachment(&self) -> Option<Attachment> {
        self.imp()
            .attachment
            .borrow()
            .as_ref()
            .and_then(|attachment| attachment.upgrade())
    }

    pub fn set_fullscreened(&self, is_fullscreened: bool) {
        if is_fullscreened == self.is_fullscreened() {
            return;
        }

        let imp = self.imp();
        imp.fullscreened.set(is_fullscreened);
        self.update_ui();
        self.notify("fullscreened");
    }

    pub fn is_fullscreened(&self) -> bool {
        self.imp().fullscreened.get()
    }

    fn set_reveal_headerbar(&self, is_reveal_headerbar: bool) {
        self.imp().flap.set_reveal_flap(is_reveal_headerbar);
    }

    fn update_ui(&self) {
        let imp = self.imp();
        let is_fullscreened = self.is_fullscreened();

        imp.flap.set_locked(!is_fullscreened);
        imp.flap.set_reveal_flap(!is_fullscreened);

        if is_fullscreened {
            imp.flap.set_fold_policy(adw::FlapFoldPolicy::Always);
            imp.picture.set_halign(gtk::Align::Fill);

            imp.fullscreen_button.set_icon_name("view-restore-symbolic");
            imp.fullscreen_button
                .set_tooltip_text(Some(&gettext("Leave fullscreen mode")));
        } else {
            imp.flap.set_fold_policy(adw::FlapFoldPolicy::Never);
            imp.picture.set_halign(gtk::Align::Center);

            imp.fullscreen_button
                .set_icon_name("view-fullscreen-symbolic");
            imp.fullscreen_button
                .set_tooltip_text(Some(&gettext("Show the current image in fullscreen mode")));
        }
    }

    fn update_content(&self) {
        if let Some(attachment) = self.attachment() {
            let file = attachment.file();
            let path = file.path().unwrap();

            let file_type = attachment.file_type();
            if file_type != FileType::Bitmap {
                log::warn!(
                    "Trying to set PictureViewer.attachment of type `{:?}`",
                    file_type
                );
                return;
            }

            spawn!(clone!(@weak self as obj => async move {
                match obj.load_texture_from_file(file).await {
                    Ok(ref texture) => {
                        obj.imp().picture.set_paintable(Some(texture));
                    }
                    Err(err) => {
                        log::error!(
                            "Failed to load texture from file `{}`: {:?}",
                            path.display(),
                            err
                        );
                    }
                }
            }));
        } else {
            self.imp().picture.set_paintable(gdk::Paintable::NONE);
        }
    }

    async fn load_texture_from_file(&self, file: gio::File) -> Result<gdk::Texture, glib::Error> {
        spawn_blocking!(move || gdk::Texture::from_file(&file)).await
    }

    fn on_exit(&self) {
        if self.is_fullscreened() {
            self.activate_action("win.toggle-fullscreen", None).unwrap();
        }

        self.emit_by_name::<()>("on-exit", &[]);
    }

    #[template_callback]
    fn on_motion(&self, _x: f64, y: f64) {
        if self.is_fullscreened() {
            let is_cursor_near_headerbar = y <= 50.0;
            self.set_reveal_headerbar(is_cursor_near_headerbar);
        }
    }

    #[template_callback]
    fn on_touch(&self) {
        if self.is_fullscreened() {
            self.set_reveal_headerbar(true);
        }
    }

    #[template_callback]
    fn on_click(&self, n_pressed: i32) {
        if n_pressed == 2 {
            self.activate_action("win.toggle-fullscreen", None).unwrap();
        }
    }
}

impl Default for PictureViewer {
    fn default() -> Self {
        Self::new()
    }
}

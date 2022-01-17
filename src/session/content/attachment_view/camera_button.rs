use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    gio,
    glib::{self, clone},
    subclass::prelude::*,
};

use crate::{session::Session, utils, widgets::Camera, Application};

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(
        resource = "/io/github/seadve/Noteworthy/ui/content-attachment-view-camera-button.ui"
    )]
    pub struct CameraButton {
        pub camera: Camera,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CameraButton {
        const NAME: &'static str = "NwtyContentAttachmentViewCameraButton";
        type Type = super::CameraButton;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("camera-button.launch", None, move |obj, _, _| {
                obj.on_launch();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CameraButton {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder(
                        "capture-done",
                        &[gio::File::static_type().into()],
                        <()>::static_type().into(),
                    )
                    .build(),
                    Signal::builder("on-launch", &[], <()>::static_type().into()).build(),
                ]
            });
            SIGNALS.as_ref()
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_signals();
        }
    }

    impl WidgetImpl for CameraButton {}
    impl BinImpl for CameraButton {}
}

glib::wrapper! {
    pub struct CameraButton(ObjectSubclass<imp::CameraButton>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible;
}

impl CameraButton {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create CameraButton")
    }

    pub fn connect_on_launch<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_local("on-launch", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            f(&obj);
            None
        })
    }

    pub fn connect_capture_done<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, &gio::File) + 'static,
    {
        self.connect_local("capture-done", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let file = values[1].get::<gio::File>().unwrap();
            f(&obj, &file);
            None
        })
    }

    fn setup_signals(&self) {
        let imp = self.imp();

        imp.camera
            .connect_capture_accept(clone!(@weak self as obj => move |_, texture| {
                let notes_dir = Session::default().directory();
                let file_path = utils::generate_unique_path(notes_dir, "Camera", Some("png"));

                if let Err(err) = texture.save_to_png(&file_path) {
                    log::error!("Failed to save texture to png: {:?}", err);
                }

                obj.emit_by_name::<()>("capture-done", &[&gio::File::for_path(&file_path)]);
            }));

        imp.camera
            .connect_on_exit(clone!(@weak self as obj => move |camera| {
                let main_window = Application::default().main_window();
                main_window.switch_to_session_page();

                // TODO Remove the page on exit.
                // The blocker is when you add, remove, then add again the same widget,
                // there will be critical errors and the actions will be disabled.
                // See https://gitlab.gnome.org/GNOME/gtk/-/issues/4421

                if let Err(err) = camera.stop() {
                    log::warn!("Failed to stop camera: {:?}", err);
                } else {
                    log::info!("Successfully stopped camera");
                }
            }));
    }

    fn on_launch(&self) {
        self.emit_by_name::<()>("on-launch", &[]);

        let imp = self.imp();

        let main_window = Application::default().main_window();

        if !main_window.has_page(&imp.camera) {
            main_window.add_page(&imp.camera);
        }

        main_window.set_visible_page(&imp.camera);

        if let Err(err) = imp.camera.start() {
            log::error!("Failed to start camera: {:?}", err);
        } else {
            log::info!("Successfully started camera");
        }
    }
}

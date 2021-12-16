use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    gio,
    glib::{self, clone, subclass::Signal},
    subclass::prelude::*,
    CompositeTemplate,
};

use crate::{utils, widgets::Camera, Application};

mod imp {
    use super::*;

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
            use once_cell::sync::Lazy;
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
        .unwrap()
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
        .unwrap()
    }

    fn setup_signals(&self) {
        let imp = imp::CameraButton::from_instance(self);

        imp.camera
            .connect_capture_accept(clone!(@weak self as obj => move |_, texture| {
                let file_path = {
                    let mut file_path = utils::default_notes_dir();
                    file_path.push(utils::generate_unique_file_name("Camera"));
                    file_path.set_extension("png");
                    file_path
                };

                texture.save_to_png(&file_path);
                obj.emit_by_name("capture-done", &[&gio::File::for_path(&file_path)]).unwrap();
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
        self.emit_by_name("on-launch", &[]).unwrap();

        let imp = imp::CameraButton::from_instance(self);

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

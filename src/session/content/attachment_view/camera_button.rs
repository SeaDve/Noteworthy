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
                vec![Signal::builder(
                    "capture-done",
                    &[gio::File::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
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
            .connect_capture_done(clone!(@weak self as obj => move |_, texture| {
                let file_name = utils::generate_unique_file_name("Camera");
                let mut file_path = utils::default_notes_dir().join(file_name);
                file_path.set_extension("png");

                texture.save_to_png(&file_path);
                obj.emit_by_name("capture-done", &[&gio::File::for_path(&file_path)]).unwrap();
            }));

        imp.camera
            .connect_on_exit(clone!(@weak self as obj => move |camera| {
                let main_window = Application::default().main_window();
                main_window.switch_to_session_page();
                main_window.remove_page(camera);

                if let Err(err) = camera.stop() {
                    log::warn!("Failed to stop camera: {:?}", err);
                }
            }));
    }

    fn on_launch(&self) {
        let imp = imp::CameraButton::from_instance(self);

        // TODO On the second add of camera page, there will be critical and the actions will be
        // disabled. See https://gitlab.gnome.org/GNOME/gtk/-/issues/4421
        let main_window = Application::default().main_window();
        main_window.add_page(&imp.camera);
        main_window.set_visible_page(&imp.camera);

        if let Err(err) = imp.camera.start() {
            log::error!("Failed to start camera: {:?}", err);
        }
    }
}

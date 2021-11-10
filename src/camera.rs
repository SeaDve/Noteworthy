use adw::{prelude::*, subclass::prelude::*};
use ashpd::{desktop::camera::CameraProxy, zbus};
use gtk::{
    glib::{self, clone, subclass::Signal},
    subclass::prelude::*,
    CompositeTemplate,
};

use crate::{spawn, widgets::CameraPaintable};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/camera.ui")]
    pub struct Camera {
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,

        pub paintable: CameraPaintable,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Camera {
        const NAME: &'static str = "NwtyCamera";
        type Type = super::Camera;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            // klass.install_action("setup.navigate-back", None, move |obj, _, _| {
            // });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Camera {
        // fn signals() -> &'static [Signal] {
        //     use once_cell::sync::Lazy;
        //     static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
        //         vec![Signal::builder(
        //             "session-setup-done",
        //             &[Session::static_type().into()],
        //             <()>::static_type().into(),
        //         )
        //         .build()]
        //     });
        //     SIGNALS.as_ref()
        // }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_paintable();
        }
    }

    impl WidgetImpl for Camera {}
    impl BinImpl for Camera {}
}

glib::wrapper! {
    pub struct Camera(ObjectSubclass<imp::Camera>)
        @extends gtk::Widget, adw::Bin;
}

impl Camera {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Camera.")
    }

    pub fn start(&self) {
        spawn!(clone!(@weak self as obj => async move {
            match obj.try_start().await {
                Ok(_) => {
                    log::info!("Camera started successfully");
                }
                Err(err) => {
                    log::error!("Failed to start camera: {:#?}", err);

                    for e in err.chain() {
                        log::error!("Failed to start camera: {:#?}", e);
                    }
                }
            }
        }));
    }

    pub fn stop(&self) {
        let imp = imp::Camera::from_instance(self);
        imp.paintable.stop();
    }

    async fn try_start(&self) -> anyhow::Result<()> {
        let connection = zbus::Connection::session().await?;
        let proxy = CameraProxy::new(&connection).await?;
        // proxy.access_camera().await?;

        // let fd = proxy.open_pipe_wire_remote().await?;

        let imp = imp::Camera::from_instance(self);
        imp.paintable.start(0)?;

        Ok(())
    }

    fn setup_paintable(&self) {
        let imp = imp::Camera::from_instance(self);
        imp.picture.set_paintable(Some(&imp.paintable));
    }

    // pub fn connect_session_setup_done<F>(&self, f: F) -> glib::SignalHandlerId
    // where
    //     F: Fn(&Self, Session) + 'static,
    // {
    //     self.connect_local("session-setup-done", true, move |values| {
    //         let obj = values[0].get::<Self>().unwrap();
    //         let session = values[1].get::<Session>().unwrap();
    //         f(&obj, session);
    //         None
    //     })
    //     .unwrap()
    // }
}

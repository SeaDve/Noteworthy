use adw::{prelude::*, subclass::prelude::*};
use ashpd::{desktop::camera::CameraProxy, zbus};
use gst::prelude::*;
use gtk::{
    gdk,
    glib::{self, clone, subclass::Signal},
    subclass::prelude::*,
    CompositeTemplate,
};
use once_cell::unsync::OnceCell;

use crate::spawn;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/camera.ui")]
    pub struct Camera {
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,

        pub pipeline: OnceCell<gst::Pipeline>,
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

            if let Err(err) = obj.setup_pipeline() {
                log::error!("Failed to setup pipeline: {:#}", err);
            }
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.pipeline
                .get()
                .unwrap()
                .set_state(gst::State::Null)
                .unwrap();
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
        let imp = imp::Camera::from_instance(self);
        let pipeline = imp.pipeline.get().unwrap();

        let bus = pipeline.bus().unwrap();
        bus.add_watch_local(
            clone!(@weak self as obj => @default-return Continue(false), move |_, message| {
                obj.handle_bus_message(message)
            }),
        )
        .unwrap();

        if let Err(err) = pipeline.set_state(gst::State::Playing) {
            log::error!("Failed to set pipeline state to Playing: {:#}", err);
        }
    }

    pub fn stop(&self) {
        let imp = imp::Camera::from_instance(self);
        let pipeline = imp.pipeline.get().unwrap();

        pipeline.set_state(gst::State::Null).unwrap();
        let bus = pipeline.bus().unwrap();
        bus.remove_watch().unwrap();
    }

    fn setup_pipeline(&self) -> anyhow::Result<()> {
        let pipeline = gst::Pipeline::new(None);

        let pipewiresrc = gst::ElementFactory::make("pipewiresrc", None)?;
        let queue = gst::ElementFactory::make("queue", None)?;
        let videoconvert = gst::ElementFactory::make("videoconvert", None)?;
        let sink = gst::ElementFactory::make("gtk4paintablesink", None)?;

        // pipewiresrc.set_property("fd", &fd.as_raw_fd())?;

        let elements = &[&pipewiresrc, &queue, &videoconvert, &sink];
        pipeline.add_many(elements)?;
        gst::Element::link_many(elements)?;

        for e in elements {
            e.sync_state_with_parent()?;
        }

        let imp = imp::Camera::from_instance(self);
        imp.pipeline.set(pipeline).unwrap();

        let paintable = sink
            .property("paintable")
            .unwrap()
            .get::<gdk::Paintable>()
            .unwrap();
        imp.picture.set_paintable(Some(&paintable));

        Ok(())
    }

    fn handle_bus_message(&self, message: &gst::Message) -> Continue {
        use gst::MessageView;

        match message.view() {
            MessageView::Error(err) => {
                log::error!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );

                self.stop();

                Continue(false)
            }
            MessageView::Element(e) => {
                if let Some(s) = e.structure() {
                    if let Ok(symbol) = s.get::<String>("symbol") {
                        self.emit_by_name("code-detected", &[&symbol]).unwrap();
                    }
                }

                Continue(true)
            }
            MessageView::StateChanged(sc) => {
                let imp = imp::Camera::from_instance(self);
                let pipeline = imp.pipeline.get().unwrap();

                if message.src().as_ref() == Some(pipeline.upcast_ref::<gst::Object>()) {
                    log::info!(
                        "Pipeline state set from {:?} -> {:?}",
                        sc.old(),
                        sc.current()
                    );
                }
                Continue(true)
            }
            _ => Continue(true),
        }
    }

    // async fn try_start(&self) -> anyhow::Result<()> {
    //     let connection = zbus::Connection::session().await?;
    //     let proxy = CameraProxy::new(&connection).await?;
    // proxy.access_camera().await?;

    // let fd = proxy.open_pipe_wire_remote().await?;

    //     let imp = imp::Camera::from_instance(self);
    //     imp.paintable.start(0)?;

    //     Ok(())
    // }

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

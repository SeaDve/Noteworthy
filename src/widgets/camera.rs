use adw::{prelude::*, subclass::prelude::*};
use anyhow::Context;
use gst::prelude::*;
use gtk::{
    gdk,
    glib::{self, clone},
    graphene,
    subclass::prelude::*,
};
use once_cell::unsync::OnceCell;

mod imp {
    use super::*;
    use glib::subclass::Signal;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Noteworthy/ui/camera.ui")]
    pub struct Camera {
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub main_control_box: TemplateChild<gtk::CenterBox>,
        #[template_child]
        pub preview_control_box: TemplateChild<gtk::CenterBox>,

        pub pipeline: OnceCell<gst::Pipeline>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Camera {
        const NAME: &'static str = "NwtyCamera";
        type Type = super::Camera;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("camera.capture", None, move |obj, _, _| {
                obj.on_capture();
            });

            klass.install_action("camera.exit", None, move |obj, _, _| {
                obj.on_exit();
            });

            klass.install_action("camera.capture-accept", None, move |obj, _, _| {
                obj.on_capture_accept();
            });

            klass.install_action("camera.capture-discard", None, move |obj, _, _| {
                obj.on_capture_discard();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Camera {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder(
                        "capture-accept",
                        &[gdk::Texture::static_type().into()],
                        <()>::static_type().into(),
                    )
                    .build(),
                    Signal::builder("on-exit", &[], <()>::static_type().into()).build(),
                ]
            });
            SIGNALS.as_ref()
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if let Err(err) = obj.setup_pipeline() {
                log::error!("Failed to setup pipeline: {:?}", err);
                // TODO handle this. Add UI error or something
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

    pub fn connect_capture_accept<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, gdk::Texture) + 'static,
    {
        self.connect_local("capture-accept", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let texture = values[1].get::<gdk::Texture>().unwrap();
            f(&obj, texture);
            None
        })
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

    pub fn start(&self) -> anyhow::Result<()> {
        let imp = imp::Camera::from_instance(self);
        let pipeline = imp.pipeline.get().unwrap();

        let bus = pipeline.bus().unwrap();
        bus.add_watch_local(
            clone!(@weak self as obj => @default-return Continue(false), move |_, message| {
                obj.handle_bus_message(message)
            }),
        )
        .unwrap();

        let res = pipeline
            .set_state(gst::State::Playing)
            .context("Failed to set pipeline state to Playing");

        if let Err(err) = res {
            self.disable_capture();
            Err(err)
        } else {
            self.enable_capture();
            Ok(())
        }
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        let imp = imp::Camera::from_instance(self);
        let pipeline = imp.pipeline.get().unwrap();

        pipeline.set_state(gst::State::Null)?;
        let bus = pipeline.bus().unwrap();
        bus.remove_watch()?;

        Ok(())
    }

    fn save_current_frame_to_texture(&self) -> gdk::Texture {
        let imp = imp::Camera::from_instance(self);
        let paintable = imp.picture.paintable().unwrap();

        let width = paintable.intrinsic_width();
        let height = paintable.intrinsic_height();

        let snapshot = gtk::Snapshot::new();
        paintable.snapshot(snapshot.upcast_ref(), width as f64, height as f64);

        let renderer = self.native().unwrap().renderer();

        let bounds = graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
        renderer.render_texture(&snapshot.to_node(), Some(&bounds))
    }

    fn enable_capture(&self) {
        self.action_set_enabled("camera.capture", true);
    }

    fn disable_capture(&self) {
        self.action_set_enabled("camera.capture", false);

        // TODO Switch to a page about error on connecting to the camera
        // Add a button too to retry reconnecting to the camera
    }

    fn setup_pipeline(&self) -> anyhow::Result<()> {
        let pipeline = gst::Pipeline::new(None);

        let pipewiresrc = gst::ElementFactory::make("pipewiresrc", None)?;
        let queue = gst::ElementFactory::make("queue", None)?;
        let videoconvert = gst::ElementFactory::make("videoconvert", None)?;
        let sink = gst::ElementFactory::make("gtk4paintablesink", None)?;

        // FIXME properly setup fd and node_id, use portal and ashpd
        // After that, also remove `--filesystem=xdg-run/pipewire-0` in flatpak manifest
        // pipewiresrc.set_property("fd", &fd.as_raw_fd())?;
        // pipewiresrc.set_property("path", node_id)?;

        let elements = &[&pipewiresrc, &queue, &videoconvert, &sink];
        pipeline.add_many(elements)?;
        gst::Element::link_many(elements)?;

        for e in elements {
            e.sync_state_with_parent()?;
        }

        let imp = imp::Camera::from_instance(self);
        imp.pipeline.set(pipeline).unwrap();

        let paintable = sink.property::<gdk::Paintable>("paintable");
        imp.picture.set_paintable(Some(&paintable));

        Ok(())
    }

    // async fn try_start(&self) -> anyhow::Result<()> {
    //     let connection = zbus::Connection::session().await?;
    //     let proxy = CameraProxy::new(&connection).await?;
    //     proxy.access_camera().await?;

    //     let fd = proxy.open_pipe_wire_remote().await?;

    //     let imp = imp::Camera::from_instance(self);
    //     imp.paintable.start(0)?;

    //     Ok(())
    // }

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

                self.stop().unwrap();
                self.disable_capture();

                Continue(false)
            }
            MessageView::StateChanged(sc) => {
                let imp = imp::Camera::from_instance(self);
                let pipeline = imp.pipeline.get().unwrap();

                if message.src().as_ref() == Some(pipeline.upcast_ref::<gst::Object>()) {
                    log::info!(
                        "Pipeline state set from `{:?}` -> `{:?}`",
                        sc.old(),
                        sc.current()
                    );
                }
                Continue(true)
            }
            _ => Continue(true),
        }
    }

    fn on_capture(&self) {
        self.stop().unwrap();

        let imp = imp::Camera::from_instance(self);
        imp.stack.set_visible_child(&imp.preview_control_box.get());
    }

    fn on_exit(&self) {
        self.emit_by_name::<()>("on-exit", &[]);
    }

    fn on_capture_accept(&self) {
        let texture = self.save_current_frame_to_texture();
        self.emit_by_name::<()>("capture-accept", &[&texture]);
        self.on_exit();
    }

    fn on_capture_discard(&self) {
        self.start().unwrap();

        let imp = imp::Camera::from_instance(self);
        imp.stack.set_visible_child(&imp.main_control_box.get());
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

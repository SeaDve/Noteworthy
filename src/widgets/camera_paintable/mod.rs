// This entire module is based on code from Decoder GPLv3
// See https://gitlab.gnome.org/World/decoder/-/blob/master/src/widgets/camera_paintable.rs

mod camera_sink;
mod frame;

use anyhow::Context;
use gst::prelude::*;
use gtk::{
    gdk,
    glib::{self, clone, subclass::Signal},
    graphene,
    prelude::*,
    subclass::prelude::*,
};
use once_cell::sync::Lazy;

use std::{cell::RefCell, os::unix::io::AsRawFd};

use self::{camera_sink::CameraSink, frame::Frame};

mod imp {
    use super::*;

    pub struct CameraPaintable {
        pub sink: CameraSink,
        pub pipeline: RefCell<Option<gst::Pipeline>>,
        pub sender: glib::Sender<camera_sink::Action>,
        pub image: RefCell<Option<gdk::Paintable>>,
        pub size: RefCell<Option<(u32, u32)>>,
        pub receiver: RefCell<Option<glib::Receiver<camera_sink::Action>>>,
    }

    impl Default for CameraPaintable {
        fn default() -> Self {
            let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

            Self {
                pipeline: RefCell::default(),
                sink: CameraSink::new(sender.clone()),
                image: RefCell::new(None),
                sender,
                receiver: RefCell::new(Some(receiver)),
                size: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CameraPaintable {
        const NAME: &'static str = "NwtyCameraPaintable";
        type Type = super::CameraPaintable;
        type ParentType = glib::Object;
        type Interfaces = (gdk::Paintable,);
    }

    impl ObjectImpl for CameraPaintable {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    "code-detected",
                    &[String::static_type().into()],
                    glib::Type::UNIT.into(),
                )
                .flags(glib::SignalFlags::RUN_FIRST)
                .build()]
            });
            SIGNALS.as_ref()
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_receiver();
        }

        fn dispose(&self, obj: &Self::Type) {
            obj.close_pipeline();
        }
    }

    impl PaintableImpl for CameraPaintable {
        fn intrinsic_height(&self, _obj: &Self::Type) -> i32 {
            if let Some((_, height)) = *self.size.borrow() {
                height as i32
            } else {
                0
            }
        }

        fn intrinsic_width(&self, _obj: &Self::Type) -> i32 {
            if let Some((width, _)) = *self.size.borrow() {
                width as i32
            } else {
                0
            }
        }

        fn snapshot(&self, obj: &Self::Type, snapshot: &gdk::Snapshot, width: f64, height: f64) {
            let snapshot = snapshot.downcast_ref::<gtk::Snapshot>().unwrap();

            obj.on_snapshot(snapshot, width, height);
        }
    }
}

glib::wrapper! {
    pub struct CameraPaintable(ObjectSubclass<imp::CameraPaintable>)
        @implements gdk::Paintable;
}

impl Default for CameraPaintable {
    fn default() -> Self {
        glib::Object::new(&[]).expect("Failed to create a CameraPaintable")
    }
}

impl CameraPaintable {
    pub fn start<F: AsRawFd>(&self, fd: F) -> anyhow::Result<()> {
        let imp = imp::CameraPaintable::from_instance(self);

        let pipeline = self.create_pipeline(fd)?;
        imp.pipeline.replace(Some(pipeline));

        let pipeline = imp.pipeline.borrow();
        let pipeline = pipeline.as_ref().unwrap();

        let bus = pipeline.bus().unwrap();
        bus.add_watch_local(
            clone!(@weak self as obj => @default-return Continue(false), move |_, message| {
                obj.handle_bus_message(message)
            }),
        )
        .unwrap();

        pipeline
            .set_state(gst::State::Playing)
            .context("Failed to set pipeline state to Playing")?;

        Ok(())
    }

    pub fn stop(&self) {
        self.close_pipeline();
    }

    fn create_pipeline<F: AsRawFd>(&self, fd: F) -> anyhow::Result<gst::Pipeline> {
        let imp = imp::CameraPaintable::from_instance(self);

        let pipeline = gst::Pipeline::new(None);

        let pipewiresrc = gst::ElementFactory::make("pipewiresrc", None)?;
        let queue = gst::ElementFactory::make("queue", None)?;
        let videoconvert = gst::ElementFactory::make("videoconvert", None)?;
        let sink = imp.sink.upcast_ref();

        // pipewiresrc.set_property("fd", &fd.as_raw_fd())?;

        let elements = &[&pipewiresrc, &queue, &videoconvert, sink];
        pipeline.add_many(elements)?;
        gst::Element::link_many(elements)?;

        Ok(pipeline)
    }

    fn setup_receiver(&self) {
        let imp = imp::CameraPaintable::from_instance(self);

        let receiver = imp.receiver.take().unwrap();
        receiver.attach(
            None,
            clone!(@weak self as obj => @default-panic, move |action| {
                let imp = imp::CameraPaintable::from_instance(&obj);

                match action {
                    camera_sink::Action::FrameChanged => {
                        if let Some(frame) = imp.sink.pending_frame() {
                            let (width, height) = (frame.width(), frame.height());

                            imp.size.replace(Some((width, height)));
                            imp.image.replace(Some(frame.into()));

                            obj.invalidate_contents();
                        }
                    }
                }

                Continue(true)
            }),
        );
    }

    fn close_pipeline(&self) {
        let imp = imp::CameraPaintable::from_instance(self);

        if let Some(pipeline) = imp.pipeline.take() {
            pipeline.set_state(gst::State::Null).unwrap();

            let bus = pipeline.bus().unwrap();
            bus.remove_watch().unwrap();
        }
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

                self.close_pipeline();

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
                let imp = imp::CameraPaintable::from_instance(self);
                let pipeline = imp.pipeline.borrow();
                let pipeline = pipeline.as_ref().unwrap();

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

    fn on_snapshot(&self, snapshot: &gtk::Snapshot, width: f64, height: f64) {
        let imp = imp::CameraPaintable::from_instance(self);

        if let Some(ref image) = *imp.image.borrow() {
            // Transformation to avoid stretching the camera. We translate and scale the image
            // under a clip.
            let clip = graphene::Rect::new(0.0, 0.0, width as f32, height as f32);

            let aspect = width / height.max(std::f64::EPSILON); // Do not divide by zero.
            let image_aspect = image.intrinsic_aspect_ratio();

            snapshot.push_clip(&clip);

            if image_aspect == 0.0 {
                image.snapshot(snapshot.upcast_ref(), width, height);
                return;
            };

            let (new_width, new_height) = match aspect <= image_aspect {
                true => (height * image_aspect, height), // Mobile view
                false => (width, width / image_aspect),  // Landscape
            };

            let p = graphene::Point::new(
                ((width - new_width) / 2.0) as f32,
                ((height - new_height) / 2.0) as f32,
            );
            snapshot.translate(&p);

            image.snapshot(snapshot.upcast_ref(), new_width, new_height);

            snapshot.pop();
        } else {
            snapshot.append_color(
                &gdk::RGBA::black(),
                &graphene::Rect::new(0f32, 0f32, width as f32, height as f32),
            );
        }
    }
}

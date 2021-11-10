use gst::subclass::prelude::*;
use gst_base::subclass::prelude::*;
use gst_video::subclass::prelude::*;
use gtk::glib;
use once_cell::sync::Lazy;

use std::sync::Mutex;

use super::Frame;

pub enum Action {
    FrameChanged,
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct CameraSink {
        pub info: Mutex<Option<gst_video::VideoInfo>>,
        pub sender: Mutex<Option<glib::Sender<Action>>>,
        pub pending_frame: Mutex<Option<Frame>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CameraSink {
        const NAME: &'static str = "NwtyCameraSink";
        type Type = super::CameraSink;
        type ParentType = gst_video::VideoSink;
    }

    impl ObjectImpl for CameraSink {}

    impl ElementImpl for CameraSink {
        fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
            static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
                gst::subclass::ElementMetadata::new(
                    "GTK Camera Sink",
                    "Sink/Camera/Video",
                    "A GTK Camera sink",
                    "Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>",
                )
            });

            Some(&*ELEMENT_METADATA)
        }

        fn pad_templates() -> &'static [gst::PadTemplate] {
            static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
                let caps = gst_video::video_make_raw_caps(&[
                    gst_video::VideoFormat::Bgra,
                    gst_video::VideoFormat::Argb,
                    gst_video::VideoFormat::Rgba,
                    gst_video::VideoFormat::Abgr,
                    gst_video::VideoFormat::Rgb,
                    gst_video::VideoFormat::Bgr,
                ])
                .any_features()
                .build();

                vec![gst::PadTemplate::new(
                    "sink",
                    gst::PadDirection::Sink,
                    gst::PadPresence::Always,
                    &caps,
                )
                .unwrap()]
            });

            PAD_TEMPLATES.as_ref()
        }
    }

    impl BaseSinkImpl for CameraSink {
        fn set_caps(
            &self,
            _element: &Self::Type,
            caps: &gst::Caps,
        ) -> Result<(), gst::LoggableError> {
            let video_info = gst_video::VideoInfo::from_caps(caps).unwrap();

            let mut info = self.info.lock().unwrap();
            info.replace(video_info);

            Ok(())
        }
    }

    impl VideoSinkImpl for CameraSink {
        fn show_frame(
            &self,
            _element: &Self::Type,
            buffer: &gst::Buffer,
        ) -> Result<gst::FlowSuccess, gst::FlowError> {
            if let Some(info) = &*self.info.lock().unwrap() {
                let frame = Frame::new(buffer, info);

                let mut last_frame = self.pending_frame.lock().unwrap();
                last_frame.replace(frame);

                let sender = self.sender.lock().unwrap();
                sender.as_ref().unwrap().send(Action::FrameChanged).unwrap();
            }

            Ok(gst::FlowSuccess::Ok)
        }
    }
}

glib::wrapper! {
    pub struct CameraSink(ObjectSubclass<imp::CameraSink>)
        @extends gst_video::VideoSink, gst_base::BaseSink, gst::Element, gst::Object;
}

unsafe impl Send for CameraSink {}
unsafe impl Sync for CameraSink {}

impl CameraSink {
    pub fn new(sender: glib::Sender<Action>) -> Self {
        let obj = glib::Object::new(&[]).expect("Failed to create a CameraSink");

        let imp = imp::CameraSink::from_instance(&obj);
        imp.sender.lock().unwrap().replace(sender);

        obj
    }

    pub fn pending_frame(&self) -> Option<Frame> {
        let imp = imp::CameraSink::from_instance(self);
        imp.pending_frame.lock().unwrap().take()
    }
}

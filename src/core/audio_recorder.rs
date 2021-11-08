use futures_channel::oneshot::{self, Receiver, Sender};
use gst_pbutils::prelude::*;
use gtk::{
    glib::{self, clone},
    subclass::prelude::*,
};
use once_cell::sync::Lazy;

use std::{
    cell::{Cell, RefCell},
    path::Path,
};

use super::AudioRecording;
use crate::spawn;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AudioRecorder {
        pub peak: Cell<f64>,

        pub recording: RefCell<Option<AudioRecording>>,
        pub pipeline: RefCell<Option<gst::Pipeline>>,
        pub sender: RefCell<Option<Sender<anyhow::Result<AudioRecording>>>>,
        pub receiver: RefCell<Option<Receiver<anyhow::Result<AudioRecording>>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioRecorder {
        const NAME: &'static str = "NwtyAudioRecorder";
        type Type = super::AudioRecorder;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for AudioRecorder {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_double(
                    "peak",
                    "Peak",
                    "Current volume peak while recording",
                    f64::MIN,
                    f64::MAX,
                    0.0,
                    glib::ParamFlags::READWRITE,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "peak" => {
                    let peak = value.get().unwrap();
                    self.peak.set(peak);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "peak" => self.peak.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct AudioRecorder(ObjectSubclass<imp::AudioRecorder>);
}

impl AudioRecorder {
    pub fn new() -> Self {
        glib::Object::new::<Self>(&[]).expect("Failed to create AudioRecorder.")
    }

    pub fn connect_peak_notify<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_notify_local(Some("peak"), move |obj, _| f(obj))
    }

    pub fn start(&self, base_path: &Path) -> Result<(), gst::StateChangeError> {
        let new_recording = AudioRecording::new(base_path);
        let pipeline = self.create_pipeline(&new_recording.path());

        let bus = pipeline.bus().unwrap();
        bus.add_watch_local(
            clone!(@weak self as obj => @default-return Continue(false), move |_, message| {
                obj.handle_bus_message(message)
            }),
        )
        .unwrap();

        let imp = imp::AudioRecorder::from_instance(self);
        imp.pipeline.replace(Some(pipeline));
        imp.recording.replace(Some(new_recording));

        let (sender, receiver) = oneshot::channel();
        imp.sender.replace(Some(sender));
        imp.receiver.replace(Some(receiver));

        let pipeline = imp.pipeline.borrow();
        let pipeline = pipeline.as_ref().unwrap();

        pipeline.set_state(gst::State::Playing)?;

        log::info!(
            "Started audio recording with device name: {}",
            pipeline
                .by_name("pulsesrc")
                .unwrap()
                .property("device")
                .unwrap()
                .get::<String>()
                .unwrap()
        );

        Ok(())
    }

    pub fn pause(&self) {
        self.pipeline().set_state(gst::State::Paused).unwrap();
    }

    pub fn resume(&self) {
        self.pipeline().set_state(gst::State::Playing).unwrap();
    }

    pub async fn stop(&self) -> anyhow::Result<AudioRecording> {
        log::info!("Sending EOS event to pipeline");
        self.pipeline().send_event(gst::event::Eos::new());

        let imp = imp::AudioRecorder::from_instance(self);
        let receiver = imp.receiver.take().unwrap();
        receiver.await.unwrap()
    }

    pub fn cancel(&self) {
        let imp = imp::AudioRecorder::from_instance(self);
        let _ = imp.sender.take();
        let _ = imp.receiver.take();

        if let Some(recording) = self.cleanup_and_take_recording() {
            spawn!(async move {
                if let Err(err) = recording.delete().await {
                    log::warn!("Failed to delete recording: {}", err);
                }
            });
        }
    }

    pub fn state(&self) -> gst::State {
        let (_ret, current, _pending) = self.pipeline().state(None);
        current
    }

    pub fn peak(&self) -> f64 {
        self.property("peak").unwrap().get().unwrap()
    }

    fn pipeline(&self) -> gst::Pipeline {
        let imp = imp::AudioRecorder::from_instance(self);
        imp.pipeline
            .borrow()
            .as_ref()
            .cloned()
            .expect("Pipeline not setup")
    }

    fn default_audio_source_name(&self) -> String {
        let res = pulsectl::controllers::SourceController::create()
            .and_then(|mut controller| controller.get_server_info())
            .and_then(|server_info| {
                server_info.default_source_name.ok_or_else(|| {
                    pulsectl::ControllerError::GetInfo("default source name not found".into())
                })
            });

        match res {
            Ok(audio_source_name) => audio_source_name,
            Err(err) => {
                log::warn!("Failed to get audio source name: {}", err);
                log::warn!("Falling back to default");
                String::new()
            }
        }
    }

    fn encodebin_profile(&self) -> gst_pbutils::EncodingContainerProfile {
        let encoding_profile = gst_pbutils::EncodingAudioProfileBuilder::new()
            .format(&gst::Caps::builder("audio/x-opus").build())
            .presence(1)
            .build()
            .unwrap();

        gst_pbutils::EncodingContainerProfileBuilder::new()
            .format(&gst::Caps::builder("application/ogg").build())
            .add_profile(&encoding_profile)
            .build()
            .unwrap()
    }

    fn create_pipeline(&self, recording_path: &Path) -> gst::Pipeline {
        let pipeline = gst::Pipeline::new(None);

        let src = gst::ElementFactory::make("pulsesrc", Some("pulsesrc")).unwrap();
        src.set_property("device", &self.default_audio_source_name())
            .unwrap();

        let audioconvert = gst::ElementFactory::make("audioconvert", None).unwrap();
        let level = gst::ElementFactory::make("level", None).unwrap();

        let encodebin = gst::ElementFactory::make("encodebin", None).unwrap();
        encodebin
            .set_property("profile", &self.encodebin_profile())
            .unwrap();

        let filesink = gst::ElementFactory::make("filesink", None).unwrap();
        filesink
            .set_property("location", recording_path.to_str().unwrap())
            .unwrap();

        pipeline
            .add_many(&[&src, &audioconvert, &level, &encodebin, &filesink])
            .unwrap();

        src.link(&audioconvert).unwrap();
        audioconvert
            .link_filtered(&level, &gst::Caps::builder("audio/x-raw").build())
            .unwrap();
        level.link(&encodebin).unwrap();
        encodebin.link(&filesink).unwrap();

        pipeline
    }

    fn cleanup_and_take_recording(&self) -> Option<AudioRecording> {
        let imp = imp::AudioRecorder::from_instance(self);

        if let Some(pipeline) = imp.pipeline.take() {
            pipeline.set_state(gst::State::Null).unwrap();

            let bus = pipeline.bus().unwrap();
            bus.remove_watch().unwrap();
        }

        imp.recording.take()
    }

    fn handle_bus_message(&self, message: &gst::Message) -> Continue {
        match message.view() {
            gst::MessageView::Element(_) => {
                let peak = message
                    .structure()
                    .unwrap()
                    .value("peak")
                    .unwrap()
                    .get::<glib::ValueArray>()
                    .unwrap()
                    .nth(0)
                    .unwrap();
                self.set_property_from_value("peak", &peak).unwrap();

                Continue(true)
            }
            gst::MessageView::Eos(_) => {
                log::info!("Eos signal received from record bus");

                let recording = self.cleanup_and_take_recording();

                let imp = imp::AudioRecorder::from_instance(self);
                let sender = imp.sender.take().unwrap();
                sender.send(Ok(recording.unwrap())).unwrap();

                Continue(false)
            }
            gst::MessageView::Error(error) => {
                log::error!(
                    "Error from record bus: {:?} (debug {:?})",
                    error.error(),
                    error
                );

                let _ = self.cleanup_and_take_recording();

                let imp = imp::AudioRecorder::from_instance(self);
                let sender = imp.sender.take().unwrap();
                sender
                    .send(Err(anyhow::anyhow!(error.error().to_string())))
                    .unwrap();

                Continue(false)
            }
            gst::MessageView::StateChanged(sc) => {
                if message.src().as_ref() == Some(self.pipeline().upcast_ref::<gst::Object>()) {
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
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new()
    }
}

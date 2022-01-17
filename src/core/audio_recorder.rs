use futures_channel::oneshot::{self, Receiver, Sender};
use gst_pbutils::prelude::*;
use gtk::{
    glib::{self, clone},
    subclass::prelude::*,
};

use std::{
    cell::{Cell, RefCell},
    path::Path,
    time::Duration,
};

use super::{AudioRecording, ClockTime};

#[derive(Debug, thiserror::Error)]
#[error("Missing element {0}")]
struct MissingElement(&'static str);

mod imp {
    use super::*;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default)]
    pub struct AudioRecorder {
        pub peak: Cell<f64>,
        pub duration: Cell<ClockTime>,

        pub recording: RefCell<Option<AudioRecording>>,
        pub pipeline: RefCell<Option<gst::Pipeline>>,
        pub source_id: RefCell<Option<glib::SourceId>>,
        pub sender: RefCell<Option<Sender<anyhow::Result<AudioRecording>>>>,
        pub receiver: RefCell<Option<Receiver<anyhow::Result<AudioRecording>>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioRecorder {
        const NAME: &'static str = "NwtyAudioRecorder";
        type Type = super::AudioRecorder;
    }

    impl ObjectImpl for AudioRecorder {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecDouble::new(
                        "peak",
                        "Peak",
                        "Current volume peak while recording",
                        f64::MIN,
                        f64::MAX,
                        0.0,
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpecBoxed::new(
                        "duration",
                        "Duration",
                        "Current duration while recording",
                        ClockTime::static_type(),
                        glib::ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "peak" => obj.peak().to_value(),
                "duration" => obj.duration().to_value(),
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

    pub fn connect_duration_notify<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_notify_local(Some("duration"), move |obj, _| f(obj))
    }

    pub fn peak(&self) -> f64 {
        self.imp().peak.get()
    }

    pub fn duration(&self) -> ClockTime {
        self.imp().duration.get()
    }

    pub fn start(&self, base_path: &Path) -> anyhow::Result<()> {
        let new_recording = AudioRecording::new(base_path);
        let pipeline = Self::default_pipeline(&new_recording.path())?;

        let bus = pipeline.bus().unwrap();
        bus.add_watch_local(
            clone!(@weak self as obj => @default-return Continue(false), move |_, message| {
                obj.handle_bus_message(message)
            }),
        )
        .unwrap();

        let imp = self.imp();
        imp.pipeline.replace(Some(pipeline));
        imp.recording.replace(Some(new_recording));

        let (sender, receiver) = oneshot::channel();
        imp.sender.replace(Some(sender));
        imp.receiver.replace(Some(receiver));

        let pipeline = imp.pipeline.borrow();
        let pipeline = pipeline.as_ref().unwrap();

        pipeline.set_state(gst::State::Playing)?;

        imp.source_id.replace(Some(glib::timeout_add_local(
            Duration::from_millis(100),
            clone!(@weak self as obj => @default-return Continue(false), move || {
                let imp = obj.imp();

                let pipeline = imp.pipeline.borrow();

                match pipeline.as_ref().unwrap().query_position::<gst::ClockTime>() {
                    Some(position) => {
                        imp.duration.set(position.into());
                        obj.notify("duration");
                    }
                    None => log::warn!("Failed to query position"),
                }

                Continue(true)
            }),
        )));

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

        let receiver = self.imp().receiver.take().unwrap();
        receiver.await.unwrap()
    }

    pub async fn cancel(&self) {
        let imp = self.imp();
        imp.sender.replace(None);
        imp.receiver.replace(None);

        if let Some(recording) = self.cleanup_and_take_recording() {
            if let Err(err) = recording.delete().await {
                log::warn!("Failed to delete recording: {:?}", err);
            }
        }
    }

    pub fn state(&self) -> gst::State {
        let (_ret, current, _pending) = self.pipeline().state(None);
        current
    }

    fn pipeline(&self) -> gst::Pipeline {
        self.imp()
            .pipeline
            .borrow()
            .as_ref()
            .cloned()
            .expect("Pipeline not setup")
    }

    fn default_audio_source_name() -> anyhow::Result<String> {
        let server_info = pulsectl::controllers::SourceController::create()?.get_server_info()?;

        server_info
            .default_source_name
            .ok_or_else(|| anyhow::anyhow!("Default audio source name not found"))
    }

    fn default_encodebin_profile() -> gst_pbutils::EncodingContainerProfile {
        let audio_caps = gst::Caps::new_simple("audio/x-opus", &[]);
        let encoding_profile = gst_pbutils::EncodingAudioProfile::builder(&audio_caps)
            .presence(1)
            .build();

        let container_caps = gst::Caps::new_simple("application/ogg", &[]);
        gst_pbutils::EncodingContainerProfile::builder(&container_caps)
            .add_profile(&encoding_profile)
            .build()
    }

    fn default_pipeline(recording_path: &Path) -> anyhow::Result<gst::Pipeline> {
        let pipeline = gst::Pipeline::new(None);

        let pulsesrc =
            gst::ElementFactory::make("pulsesrc", None).map_err(|_| MissingElement("pulsesrc"))?;
        let audioconvert = gst::ElementFactory::make("audioconvert", None)
            .map_err(|_| MissingElement("audioconvert"))?;
        let level =
            gst::ElementFactory::make("level", None).map_err(|_| MissingElement("level"))?;
        let encodebin = gst::ElementFactory::make("encodebin", None)
            .map_err(|_| MissingElement("encodebin"))?;
        let filesink =
            gst::ElementFactory::make("filesink", None).map_err(|_| MissingElement("filesink"))?;

        match Self::default_audio_source_name() {
            Ok(ref audio_source_name) => {
                log::info!(
                    "Pipeline setup with pulsesrc device name `{}`",
                    audio_source_name
                );
                pulsesrc.set_property("device", audio_source_name);
            }
            Err(err) => log::warn!("Failed to set pulsesrc device: {:?}", err),
        }

        encodebin.set_property("profile", &Self::default_encodebin_profile());
        filesink.set_property("location", recording_path.to_str().unwrap());

        let elements = [&pulsesrc, &audioconvert, &level, &encodebin, &filesink];
        pipeline.add_many(&elements)?;

        pulsesrc.link(&audioconvert)?;
        audioconvert.link_filtered(&level, &gst::Caps::builder("audio/x-raw").build())?;
        level.link(&encodebin)?;
        encodebin.link(&filesink)?;

        for e in elements {
            e.sync_state_with_parent()?;
        }

        Ok(pipeline)
    }

    fn cleanup_and_take_recording(&self) -> Option<AudioRecording> {
        let imp = self.imp();

        if let Some(pipeline) = imp.pipeline.take() {
            if let Some(source_id) = imp.source_id.take() {
                source_id.remove();
            }

            pipeline.set_state(gst::State::Null).unwrap();

            let bus = pipeline.bus().unwrap();
            bus.remove_watch().unwrap();
        }

        imp.recording.take()
    }

    fn handle_bus_message(&self, message: &gst::Message) -> Continue {
        use gst::MessageView;

        match message.view() {
            MessageView::Element(_) => {
                let peak = message
                    .structure()
                    .unwrap()
                    .value("peak")
                    .unwrap()
                    .get::<glib::ValueArray>()
                    .unwrap()
                    .nth(0)
                    .unwrap()
                    .get::<f64>()
                    .unwrap();

                self.imp().peak.set(peak);
                self.notify("peak");

                Continue(true)
            }
            MessageView::Eos(_) => {
                log::info!("Eos signal received from record bus");

                let recording = self.cleanup_and_take_recording();

                let sender = self.imp().sender.take().unwrap();
                sender.send(Ok(recording.unwrap())).unwrap();

                Continue(false)
            }
            MessageView::Error(err) => {
                log::error!("Error from record bus: {:?} (debug {:?})", err.error(), err);

                let _recording = self.cleanup_and_take_recording();

                let sender = self.imp().sender.take().unwrap();
                sender.send(Err(err.error().into())).unwrap();

                Continue(false)
            }
            MessageView::StateChanged(sc) => {
                if message.src().as_ref() == Some(self.pipeline().upcast_ref::<gst::Object>()) {
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
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new()
    }
}

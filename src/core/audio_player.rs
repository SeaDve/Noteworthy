// Rust rewrite of gstplayer.py from GNOME Music (GPLv2)
// Modified to remove features that will be unused
// See https://gitlab.gnome.org/GNOME/gnome-music/-/blob/master/gnomemusic/gstplayer.py

use gst::prelude::*;
use gtk::{
    glib::{self, clone, GEnum},
    subclass::prelude::*,
};
use once_cell::{sync::Lazy, unsync::OnceCell};

use std::{cell::Cell, time::Duration};

#[derive(Debug, PartialEq, Clone, Copy, GEnum)]
#[genum(type_name = "AudioPlayerPlaybackState")]
pub enum PlaybackState {
    Stopped,
    Loading,
    Paused,
    Playing,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self::Stopped
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AudioPlayer {
        pub player: OnceCell<gst::Pipeline>,

        pub state: Cell<PlaybackState>,
        pub duration: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioPlayer {
        const NAME: &'static str = "NwtyAudioPlayer";
        type Type = super::AudioPlayer;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for AudioPlayer {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_enum(
                        "state",
                        "State",
                        "Current state of the player",
                        PlaybackState::static_type(),
                        PlaybackState::default() as i32,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_string(
                        "uri",
                        "Uri",
                        "Current uri being played in the player",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpec::new_uint64(
                        "position",
                        "Position",
                        "Current position in the player",
                        0,
                        u64::MAX,
                        0,
                        glib::ParamFlags::READABLE,
                    ),
                    glib::ParamSpec::new_int(
                        "duration",
                        "Duration",
                        "Duration of what is playing in the player",
                        -1,
                        i32::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "state" => {
                    let state = value.get().unwrap();
                    obj.set_state(state);
                }
                "uri" => {
                    let uri = value.get().unwrap();
                    obj.set_uri(uri);
                }
                "duration" => {
                    let duration = value.get().unwrap();
                    obj.set_duration(duration);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "state" => obj.state().to_value(),
                "uri" => obj.uri().to_value(),
                "position" => obj.position().to_value(),
                "duration" => obj.duration().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_player();
        }
    }
}

glib::wrapper! {
    pub struct AudioPlayer(ObjectSubclass<imp::AudioPlayer>);
}

impl AudioPlayer {
    pub fn new() -> Self {
        glib::Object::new::<Self>(&[]).expect("Failed to create AudioPlayer.")
    }

    pub fn set_state(&self, state: PlaybackState) {
        let player = self.player();

        match state {
            PlaybackState::Stopped => {
                player.set_state(gst::State::Null).unwrap();
                log::info!("Player state changed to Stopped");

                // Changing the state to NULL flushes the pipeline.
                // Thus, the change message never arrives.
                let imp = imp::AudioPlayer::from_instance(self);
                imp.state.set(state);
                self.notify("state");
            }
            PlaybackState::Loading => {
                player.set_state(gst::State::Ready).unwrap();
            }
            PlaybackState::Paused => {
                player.set_state(gst::State::Paused).unwrap();
            }
            PlaybackState::Playing => {
                player.set_state(gst::State::Playing).unwrap();
            }
        }
    }

    pub fn state(&self) -> PlaybackState {
        let imp = imp::AudioPlayer::from_instance(self);
        imp.state.get()
    }

    pub fn set_uri(&self, uri: &str) {
        self.player().set_property("uri", uri).unwrap();
        self.notify("uri");
    }

    pub fn uri(&self) -> String {
        self.player()
            .property("uri")
            .unwrap()
            .get()
            .unwrap_or_default()
    }

    pub fn seek(&self, position: u64) {
        let flags = gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT;
        let clock_time_position = gst::ClockTime::from_seconds(position);
        if let Err(err) = self.player().seek_simple(flags, clock_time_position) {
            log::error!("Failed to seek at pos {}: {}", position, err);
        }
    }

    pub fn position(&self) -> u64 {
        let clock_time: Option<gst::ClockTime> = self.player().query_position();
        clock_time.map_or(0, gst::ClockTime::seconds)
    }

    pub fn set_duration(&self, duration: i32) {
        let imp = imp::AudioPlayer::from_instance(self);
        imp.duration.replace(duration);
        self.notify("duration");
    }

    pub fn duration(&self) -> i32 {
        if self.state() == PlaybackState::Stopped {
            return -1;
        }

        let imp = imp::AudioPlayer::from_instance(self);
        imp.duration.get()
    }

    pub fn load(&self, uri: &str) {
        self.set_state(PlaybackState::Loading);
        self.set_uri(uri);
    }

    pub fn play(&self) {
        self.set_state(PlaybackState::Playing);
    }

    pub fn load_and_play(&self, uri: &str) {
        self.load(uri);
        self.play();
    }

    pub fn stop(&self) {
        self.set_state(PlaybackState::Stopped);
    }

    fn player(&self) -> gst::Pipeline {
        let imp = imp::AudioPlayer::from_instance(self);
        imp.player.get().expect("Player not setup").clone()
    }

    fn setup_player(&self) {
        let imp = imp::AudioPlayer::from_instance(self);

        let player: gst::Pipeline = gst::ElementFactory::make("playbin3", Some("playbin"))
            .unwrap()
            .downcast()
            .unwrap();

        let bus = player.bus().unwrap();
        bus.add_watch_local(
            clone!(@weak self as obj => @default-return Continue(true), move |_, message| {
                obj.handle_bus_message(message)
            }),
        )
        .unwrap();

        imp.player.set(player).unwrap();
    }

    fn handle_bus_message(&self, message: &gst::Message) -> glib::Continue {
        use gst::MessageView::*;
        match message.view() {
            Error(message) => self.on_bus_error(message),
            Eos(_) => self.on_bus_eos(),
            StateChanged(message) => self.on_state_changed(message),
            StreamStart(_) => self.on_bus_stream_start(),
            _ => (),
        }

        glib::Continue(true)
    }

    fn query_duration(&self) {
        let duration = self
            .player()
            .query_duration::<gst::ClockTime>()
            .map_or(-1, |d| d.seconds() as i32);
        self.set_duration(duration);
    }

    fn on_bus_error(&self, message: gst::message::Error) {
        let error = message.error();
        let debug = message.debug();

        log::warn!(
            "Error from element {}: {:?}",
            message.src().unwrap().name(),
            error
        );

        if let Some(debug) = debug {
            log::warn!("Debug info: {}", debug);
        }

        log::warn!("Error while playing audio with uri: {}", self.uri());

        self.set_state(PlaybackState::Stopped);
    }

    fn on_bus_eos(&self) {
        self.set_state(PlaybackState::Stopped);
    }

    fn on_state_changed(&self, message: gst::message::StateChanged) {
        if message.src().as_ref() != Some(self.player().upcast_ref::<gst::Object>()) {
            return;
        }

        let old_state = message.old();
        let new_state = message.current();

        log::info!("Player state changed: {:?} -> {:?}", old_state, new_state);

        let state = match new_state {
            gst::State::Null => PlaybackState::Stopped,
            gst::State::Ready => PlaybackState::Loading,
            gst::State::Paused => PlaybackState::Paused,
            gst::State::Playing => PlaybackState::Playing,
            _ => return,
        };

        let imp = imp::AudioPlayer::from_instance(self);
        imp.state.set(state);
        self.notify("state");
    }

    fn on_bus_stream_start(&self) {
        // Delay the signalling slightly or the new duration will not
        // have been set yet.
        glib::timeout_add_local_once(
            Duration::from_millis(1),
            clone!(@weak self as obj => move || {
                obj.query_duration();
            }),
        );
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

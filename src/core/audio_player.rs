// Rust rewrite of gstplayer.py from GNOME Music (GPLv2)
// Modified to remove features that will be unused
// See https://gitlab.gnome.org/GNOME/gnome-music/-/blob/master/gnomemusic/gstplayer.py

use gst::prelude::*;
use gtk::{
    glib::{self, clone, GEnum},
    subclass::prelude::*,
};
use once_cell::{sync::Lazy, unsync::OnceCell};

use std::cell::{Cell, RefCell};

use crate::spawn_blocking;

#[derive(Debug, PartialEq, Clone, Copy, GEnum)]
#[genum(type_name = "AudioPlayerPlaybackState")]
pub enum PlaybackState {
    Stopped,
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
        pub uri: RefCell<String>,
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
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "state" => obj.state().to_value(),
                "uri" => obj.uri().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.setup_player();
        }
    }

    impl Drop for AudioPlayer {
        fn drop(&mut self) {
            self.player
                .get()
                .unwrap()
                .set_state(gst::State::Null)
                .unwrap();
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

    pub fn connect_state_notify<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, &glib::ParamSpec) + 'static,
    {
        self.connect_notify_local(Some("state"), f)
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

        let imp = imp::AudioPlayer::from_instance(self);
        imp.uri.replace(uri.to_owned());

        self.notify("uri");
    }

    pub fn uri(&self) -> String {
        let imp = imp::AudioPlayer::from_instance(self);
        imp.uri.borrow().clone()
    }

    pub fn seek(&self, position: u64) {
        let flags = gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT;
        let clock_time_position = gst::ClockTime::from_seconds(position);
        if let Err(err) = self.player().seek_simple(flags, clock_time_position) {
            log::error!("Failed to seek at pos {}: {}", position, err);
        }
    }

    pub fn query_position(&self) -> anyhow::Result<u64> {
        match self.player().query_position::<gst::ClockTime>() {
            Some(clock_time) => Ok(clock_time.seconds()),
            None => anyhow::bail!("Failed to query position"),
        }
    }

    pub async fn duration(&self) -> anyhow::Result<u64> {
        let uri = self.uri();

        let discover_info = spawn_blocking!(move || {
            let timeout = gst::ClockTime::from_seconds(10);
            let discoverer = gst_pbutils::Discoverer::new(timeout).unwrap();
            discoverer.discover_uri(&uri)
        })
        .await?;

        Ok(discover_info.duration().map_or(0, gst::ClockTime::seconds))
    }

    pub fn play(&self) {
        self.set_state(PlaybackState::Playing);
    }

    pub fn pause(&self) {
        self.set_state(PlaybackState::Paused);
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
            clone!(@weak self as obj => @default-return Continue(false), move |_, message| {
                obj.handle_bus_message(message)
            }),
        )
        .unwrap();

        imp.player.set(player).unwrap();
    }

    fn handle_bus_message(&self, message: &gst::Message) -> Continue {
        use gst::MessageView::*;
        match message.view() {
            Error(message) => self.on_bus_error(message),
            Eos(_) => self.on_bus_eos(),
            StateChanged(message) => self.on_state_changed(message),
            _ => (),
        }

        Continue(true)
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
            gst::State::Paused => PlaybackState::Paused,
            gst::State::Playing => PlaybackState::Playing,
            _ => return,
        };

        let imp = imp::AudioPlayer::from_instance(self);
        imp.state.set(state);
        self.notify("state");
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

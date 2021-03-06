// Rust rewrite of gstplayer.py from GNOME Music (GPLv2)
// Modified to remove features that will be unused
// See https://gitlab.gnome.org/GNOME/gnome-music/-/blob/master/gnomemusic/gstplayer.py

use gst::prelude::*;
use gtk::{
    glib::{self, clone},
    subclass::prelude::*,
};
use once_cell::unsync::OnceCell;

use std::cell::{Cell, RefCell};

use super::ClockTime;
use crate::spawn_blocking;

#[derive(Debug, Clone, Copy, PartialEq, glib::Enum)]
#[enum_type(name = "AudioPlayerPlaybackState")]
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
    use once_cell::sync::Lazy;

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
    }

    impl ObjectImpl for AudioPlayer {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecEnum::new(
                        "state",
                        "State",
                        "Current state of the player",
                        PlaybackState::static_type(),
                        PlaybackState::default() as i32,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                    ),
                    glib::ParamSpecString::new(
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
        glib::Object::new(&[]).expect("Failed to create AudioPlayer.")
    }

    pub fn connect_state_notify<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self) + 'static,
    {
        self.connect_notify_local(Some("state"), move |obj, _| f(obj))
    }

    pub fn set_state(&self, state: PlaybackState) {
        if let Err(err) = self.set_state_inner(state) {
            log::error!("Failed to set state to `{:?}`: {:?}", state, err);
            // TODO propagate this error to show user facing errors
        }
    }

    pub fn state(&self) -> PlaybackState {
        self.imp().state.get()
    }

    pub fn set_uri(&self, uri: &str) {
        self.player().set_property("uri", uri);

        self.imp().uri.replace(uri.to_owned());

        self.notify("uri");
    }

    pub fn uri(&self) -> String {
        self.imp().uri.borrow().clone()
    }

    pub fn seek(&self, position: ClockTime) {
        let position: gst::ClockTime = position
            .try_into()
            .expect("Position in nanos cannot be above std::u64::MAX");

        let flags = gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT;
        if let Err(err) = self.player().seek_simple(flags, position) {
            log::error!("Failed to seek at pos `{}`: {:?}", position, err);
        }
    }

    pub fn query_position(&self) -> anyhow::Result<ClockTime> {
        match self.player().query_position::<gst::ClockTime>() {
            Some(clock_time) => Ok(clock_time.into()),
            None => Err(anyhow::anyhow!("Failed to query position")),
        }
    }

    pub async fn duration(&self) -> anyhow::Result<ClockTime> {
        let uri = self.uri();

        let discover_info = spawn_blocking!(move || {
            let timeout = gst::ClockTime::from_seconds(10);
            let discoverer = gst_pbutils::Discoverer::new(timeout).unwrap();
            discoverer.discover_uri(&uri)
        })
        .await?;

        Ok(discover_info
            .duration()
            .map_or(ClockTime::ZERO, |ct| ct.into()))
    }

    fn set_state_inner(&self, state: PlaybackState) -> anyhow::Result<()> {
        let player = self.player();

        match state {
            PlaybackState::Stopped => {
                player.set_state(gst::State::Null)?;
                log::info!("Player state changed to Stopped");

                // Changing the state to NULL flushes the pipeline.
                // Thus, the change message never arrives.
                self.imp().state.set(state);
                self.notify("state");
            }
            PlaybackState::Loading => {
                player.set_state(gst::State::Ready)?;
            }
            PlaybackState::Paused => {
                player.set_state(gst::State::Paused)?;
            }
            PlaybackState::Playing => {
                player.set_state(gst::State::Playing)?;
            }
        }

        Ok(())
    }

    fn player(&self) -> gst::Pipeline {
        self.imp().player.get().expect("Player not setup").clone()
    }

    fn setup_player(&self) {
        let player = gst::ElementFactory::make("playbin3", None)
            .unwrap()
            .downcast::<gst::Pipeline>()
            .unwrap();

        let bus = player.bus().unwrap();
        bus.add_watch_local(
            clone!(@weak self as obj => @default-return Continue(false), move |_, message| {
                obj.handle_bus_message(message)
            }),
        )
        .unwrap();

        self.imp().player.set(player).unwrap();
    }

    fn handle_bus_message(&self, message: &gst::Message) -> Continue {
        use gst::MessageView;

        match message.view() {
            MessageView::Error(ref message) => self.on_bus_error(message),
            MessageView::Eos(_) => self.on_bus_eos(),
            MessageView::StateChanged(ref message) => self.on_state_changed(message),
            _ => (),
        }

        Continue(true)
    }

    fn on_bus_error(&self, message: &gst::message::Error) {
        let error = message.error();
        let debug = message.debug();

        log::warn!(
            "Error from element `{}`: {:?}",
            message.src().unwrap().name(),
            error
        );

        if let Some(debug) = debug {
            log::warn!("Debug info: {}", debug);
        }

        log::warn!("Error while playing audio with uri `{}`", self.uri());

        self.set_state(PlaybackState::Stopped);
    }

    fn on_bus_eos(&self) {
        self.set_state(PlaybackState::Stopped);
    }

    fn on_state_changed(&self, message: &gst::message::StateChanged) {
        if message.src().as_ref() != Some(self.player().upcast_ref::<gst::Object>()) {
            return;
        }

        let old_state = message.old();
        let new_state = message.current();

        log::info!(
            "Player state changed: `{:?}` -> `{:?}`",
            old_state,
            new_state
        );

        let state = match new_state {
            gst::State::Null => PlaybackState::Stopped,
            gst::State::Ready => PlaybackState::Loading,
            gst::State::Paused => PlaybackState::Paused,
            gst::State::Playing => PlaybackState::Playing,
            _ => return,
        };

        self.imp().state.set(state);
        self.notify("state");
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

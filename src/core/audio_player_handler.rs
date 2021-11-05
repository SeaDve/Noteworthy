use adw::subclass::prelude::*;
use gtk::{
    glib::{self, clone},
    prelude::*,
};

use std::{cell::RefCell, collections::HashMap};

use super::{AudioPlayer, PlaybackState};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AudioPlayerHandler {
        pub list: RefCell<HashMap<AudioPlayer, glib::SignalHandlerId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioPlayerHandler {
        const NAME: &'static str = "NwtyAudioPlayerHandler";
        type Type = super::AudioPlayerHandler;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for AudioPlayerHandler {}
}

glib::wrapper! {
    pub struct AudioPlayerHandler(ObjectSubclass<imp::AudioPlayerHandler>);
}

impl AudioPlayerHandler {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create AudioPlayerHandler.")
    }

    pub fn append(&self, audio_player: AudioPlayer) {
        let imp = imp::AudioPlayerHandler::from_instance(self);

        let handler_id =
            audio_player.connect_state_notify(clone!(@weak self as obj => move |audio_player,_| {
                if audio_player.state() == PlaybackState::Playing {
                    obj.stop_all_except(audio_player);
                    log::info!("Stopping all except: {}", audio_player.uri());
                }
            }));

        let mut list = imp.list.borrow_mut();
        list.insert(audio_player, handler_id);
    }

    pub fn remove(&self, audio_player: &AudioPlayer) {
        let imp = imp::AudioPlayerHandler::from_instance(self);

        let mut list = imp.list.borrow_mut();
        let handler_id = list
            .remove(audio_player)
            .expect("Trying to remove audio_player that is not handled by this");
        audio_player.disconnect(handler_id);
    }

    pub fn stop_all(&self) {
        let imp = imp::AudioPlayerHandler::from_instance(self);

        for audio_player in imp.list.borrow().keys() {
            audio_player.stop();
        }
    }

    fn stop_all_except(&self, exception: &AudioPlayer) {
        let imp = imp::AudioPlayerHandler::from_instance(self);

        for audio_player in imp.list.borrow().keys() {
            if audio_player != exception {
                audio_player.stop();
            }
        }
    }
}

impl Default for AudioPlayerHandler {
    fn default() -> Self {
        Self::new()
    }
}

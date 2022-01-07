use gst_pbutils::prelude::*;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use once_cell::unsync::OnceCell;

use std::path::{Path, PathBuf};

use crate::utils;

mod imp {
    use super::*;
    use once_cell::sync::Lazy;

    #[derive(Debug, Default)]
    pub struct AudioRecording {
        pub file: OnceCell<gio::File>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AudioRecording {
        const NAME: &'static str = "NwtyAudioRecording";
        type Type = super::AudioRecording;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for AudioRecording {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_object(
                    "file",
                    "File",
                    "File where the recording is saved",
                    gio::File::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
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
                "file" => {
                    let file = value.get().unwrap();
                    self.file.set(file).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "file" => self.file.get().unwrap().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct AudioRecording(ObjectSubclass<imp::AudioRecording>);
}

impl AudioRecording {
    pub fn new(base_path: &Path) -> Self {
        let file_path = utils::generate_unique_path(base_path, "AudioRecording", Some("ogg"));
        let file = gio::File::for_path(&file_path);

        glib::Object::new::<Self>(&[("file", &file)]).expect("Failed to create AudioRecording.")
    }

    pub fn path(&self) -> PathBuf {
        self.file().path().unwrap()
    }

    pub fn file(&self) -> gio::File {
        self.property("file").unwrap().get().unwrap()
    }

    pub async fn delete(&self) -> anyhow::Result<()> {
        self.file()
            .delete_async_future(glib::PRIORITY_DEFAULT_IDLE)
            .await?;

        Ok(())
    }
}

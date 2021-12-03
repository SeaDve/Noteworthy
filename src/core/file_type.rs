use gtk::{gio, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Bitmap,
    Audio,
    Markdown,
    Unknown,
}

impl FileType {
    pub fn for_file(file: &gio::File) -> Self {
        let res = file.query_info(
            &gio::FILE_ATTRIBUTE_STANDARD_CONTENT_TYPE,
            gio::FileQueryInfoFlags::NONE,
            None::<&gio::Cancellable>,
        );

        match res {
            Ok(file_info) => {
                let mime_type = file_info.content_type().unwrap();
                log::info!("Found mimetype of {} for file {}", mime_type, file.uri());

                match mime_type.as_str() {
                    "image/png" | "image/jpeg" => Self::Bitmap,
                    "audio/x-vorbis+ogg" | "audio/x-opus+ogg" => Self::Audio,
                    "text/markdown" => Self::Markdown,
                    _ => Self::Unknown,
                }
            }
            Err(err) => {
                log::warn!("Failed to query info for file {}: {}", file.uri(), err);
                Self::Unknown
            }
        }
    }
}

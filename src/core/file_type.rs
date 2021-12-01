use gtk::gio;

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Bitmap,
    Audio,
    Markdown,
    Unknown,
}

impl FileType {
    pub fn for_path(path: &Path) -> Self {
        use gtk::prelude::*;
        dbg!(gio::File::for_path(path)
            .query_info(
                "standard::*",
                gio::FileQueryInfoFlags::NONE,
                None::<&gio::Cancellable>
            )
            .map(|i| i.content_type()));

        let (mime_type, is_certain) = gio::content_type_guess(Some(&path.to_string_lossy()), &[]);

        log::info!(
            "Mimetype of {} for path {}; is certain: {}",
            mime_type,
            path.display(),
            is_certain
        );

        match mime_type.as_str() {
            "image/png" | "image/jpeg" => Self::Bitmap,
            "audio/ogg" => Self::Audio,
            "text/markdown" => Self::Markdown,
            _ => Self::Unknown,
        }
    }
}

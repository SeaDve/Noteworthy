use gtk::glib;

#[derive(Debug)]
pub enum Error {
    Provider(String),
    Note(String),
    Glib(glib::Error),
    String(std::string::FromUtf8Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Provider(e) => f.write_str(&format!("NoteProviderError: {}", e)),
            Self::Note(e) => f.write_str(&format!("NoteError: {}", e)),
            Self::Glib(e) => f.write_str(&format!("GlibError: {}", e)),
            Self::String(e) => f.write_str(&format!("FromUtf8Error: {}", e)),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Provider(error.to_string())
    }
}

impl From<glib::Error> for Error {
    fn from(error: glib::Error) -> Self {
        Error::Glib(error)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Self {
        Error::String(error)
    }
}

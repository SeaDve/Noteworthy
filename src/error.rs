// TODO remove this file, since it is not currently used

use gtk::glib;

#[derive(Debug)]
pub enum Error {
    Str(std::str::Utf8Error),
    Io(std::io::Error),
    Glib(glib::Error),
    Yaml(serde_yaml::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(e) => f.write_str(&format!("Utf8Error: {}", e)),
            Self::Io(e) => f.write_str(&format!("IoError: {}", e)),
            Self::Glib(e) => f.write_str(&format!("GlibError: {}", e)),
            Self::Yaml(e) => f.write_str(&format!("YamlError: {}", e)),
        }
    }
}

impl From<glib::Error> for Error {
    fn from(error: glib::Error) -> Self {
        Error::Glib(error)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        Error::Str(error)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Error::Yaml(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

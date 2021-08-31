#[derive(Debug)]
pub enum Error {
    Provider(String),
    Note(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Provider(e) => f.write_str(&format!("NoteProviderError: {}", e)),
            Self::Note(e) => f.write_str(&format!("NoteError: {}", e)),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Provider(error.to_string())
    }
}

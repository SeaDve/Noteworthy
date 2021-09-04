use chrono::{DateTime, Local};
use gtk::glib::{self, GBoxed};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, GBoxed, Deserialize)]
#[gboxed(type_name = "NwtyDate")]
pub struct Date(DateTime<Local>);

impl Default for Date {
    fn default() -> Self {
        Self(chrono::offset::Local::now())
    }
}

impl Serialize for Date {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.format("%B %d %Y %H:%M").to_string())
    }
}

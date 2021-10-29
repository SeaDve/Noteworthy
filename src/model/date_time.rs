use chrono::Local;
use gtk::glib::{self, GBoxed};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, GBoxed, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[gboxed(type_name = "NwtyDateTime")]
#[serde(transparent)]
pub struct DateTime(chrono::DateTime<Local>);

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.format("%B %d %Y %H:%M:%S").fmt(f)
    }
}

impl Default for DateTime {
    fn default() -> Self {
        Self::now()
    }
}

impl DateTime {
    pub fn now() -> Self {
        Self(Local::now())
    }

    pub fn fuzzy_display(&self) -> String {
        let now = Local::now();

        let is_today = now.date() == self.0.date();
        let duration = now.signed_duration_since(self.0);

        let hours_difference = duration.num_hours();
        let week_difference = duration.num_weeks();

        if is_today {
            self.0.format("%I∶%M") // 08:10
        } else if hours_difference <= 30 {
            self.0.format("yesterday")
        } else if week_difference <= 52 {
            self.0.format("%b %d") // Sep 03
        } else {
            self.0.format("%b %d %Y") // Sep 03 1920
        }
        .to_string()
    }
}
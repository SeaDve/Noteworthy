use chrono::{DateTime, Local};
use gtk::glib::{self, GBoxed};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, GBoxed, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[gboxed(type_name = "NwtyDate")]
#[serde(transparent)]
pub struct Date(DateTime<Local>);

impl Default for Date {
    fn default() -> Self {
        Self::now()
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.format("%B %d %Y %H:%M:%S").to_string())
    }
}

impl Date {
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

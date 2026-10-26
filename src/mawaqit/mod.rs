//! Adapter for [Mawaqit](https://mawaqit.net), the mosque timetable platform.
//!
//! Every public mosque page embeds its full configuration as `confData`: a 12-month
//! calendar of the six daily times, iqama rules, Jumu'a settings and the time zone.
//! This module extracts and parses it, and turns user-pasted links into mosque slugs.

mod configuration;
mod extract;
mod link;

pub use configuration::{Month, MosqueConfiguration};
pub use extract::extract_conf_data;
pub use link::mosque_slug;

/// The public page of a mosque, e.g. `https://mawaqit.net/en/assunna-hannover`.
pub fn page_url(slug: &str) -> String {
    format!("https://mawaqit.net/en/{slug}")
}

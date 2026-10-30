//! Adapter for [Mawaqit](https://mawaqit.net), the mosque timetable platform.
//!
//! Every public mosque page embeds its full configuration as `confData`: a 12-month
//! calendar of the six daily times, iqama rules, Jumu'a settings and the time zone.
//! This module extracts and parses it, and turns user-pasted links into mosque slugs.
//! Mosque search is sans-IO too: [`MosqueSearchRequest`] says what to send and
//! [`parse_search_results`] reads the response.

mod configuration;
mod extract;
mod link;
mod search;

pub use configuration::{Month, MosqueConfiguration};
pub use extract::extract_conf_data;
pub use link::mosque_slug;
pub use search::{HttpHeader, MosqueSearchRequest, MosqueSearchResult, parse_search_results};

/// The public page of a mosque, e.g. `https://mawaqit.net/en/assunna-hannover`.
pub fn page_url(slug: &str) -> String {
    format!("https://mawaqit.net/en/{slug}")
}

//! Foreign-language interface (UniFFI). The generated bindings expose the same types
//! as the Rust API; free functions cover what records can't have as methods.

use std::sync::Arc;
use std::time::SystemTime;

use jiff::Timestamp;

use crate::{
    Error, MosqueTimetable, Prayer, PrayerEvent, PrayerStatus, PrayerTime, countdown, mawaqit,
};

// Timestamps cross the boundary as the platform's date type (`Date` in Swift).
uniffi::custom_type!(Timestamp, SystemTime, {
    remote,
    lower: |timestamp| SystemTime::from(timestamp),
    try_lift: |time| Ok(Timestamp::try_from(time)?),
});

#[uniffi::export]
impl MosqueTimetable {
    /// Builds a timetable from a `confData` JSON object.
    #[uniffi::constructor(name = "fromConfigurationJson")]
    pub fn ffi_from_configuration_json(
        configuration_json: String,
        fetched_at: Timestamp,
    ) -> Result<Arc<Self>, Error> {
        Self::from_configuration_json(&configuration_json, fetched_at).map(Arc::new)
    }

    /// The mosque's IANA time zone, e.g. `Europe/Berlin`.
    pub fn time_zone_identifier(&self) -> Option<String> {
        self.time_zone().iana_name().map(str::to_owned)
    }
}

/// The `confData` JSON object embedded in a Mawaqit mosque page.
#[uniffi::export]
pub fn extract_conf_data(html: String) -> Result<String, Error> {
    mawaqit::extract_conf_data(&html).map(str::to_owned)
}

/// The mosque slug in a pasted Mawaqit link (or a bare slug), if valid.
#[uniffi::export]
pub fn mosque_slug(input: String) -> Option<String> {
    mawaqit::mosque_slug(&input)
}

/// The public Mawaqit page of a mosque.
#[uniffi::export]
pub fn mawaqit_page_url(slug: String) -> String {
    mawaqit::page_url(&slug)
}

/// Short countdown text: `1h 23m`, `23m`, `45s`.
#[uniffi::export]
pub fn countdown_compact(seconds: f64) -> String {
    countdown::compact(seconds)
}

/// Clock countdown text: `1:23:45` or `23:45`.
#[uniffi::export]
pub fn countdown_clock(seconds: f64) -> String {
    countdown::clock(seconds)
}

/// Seconds until the compact countdown text changes.
#[uniffi::export]
pub fn countdown_time_until_change(seconds: f64) -> f64 {
    countdown::time_until_compact_change(seconds)
}

/// How long a status bar label for `status` stays valid.
#[uniffi::export]
pub fn status_bar_refresh_interval(
    status: Option<PrayerStatus>,
    now: Timestamp,
    shows_countdown: bool,
    max_interval: f64,
) -> f64 {
    countdown::refresh_interval(status.as_ref(), now, shows_countdown, max_interval)
}

/// Seconds until the status's target, never negative.
#[uniffi::export]
pub fn prayer_status_remaining(status: PrayerStatus, now: Timestamp) -> f64 {
    status.remaining(now)
}

/// Fraction of the status's interval that has elapsed, from 0 to 1.
#[uniffi::export]
pub fn prayer_status_progress(status: PrayerStatus, now: Timestamp) -> f64 {
    status.progress(now)
}

/// Transliterated name, e.g. `Fajr`.
#[uniffi::export]
pub fn prayer_name(prayer: Prayer) -> String {
    prayer.name().to_owned()
}

/// Arabic name, e.g. `الفجر`.
#[uniffi::export]
pub fn prayer_arabic_name(prayer: Prayer) -> String {
    prayer.arabic_name().to_owned()
}

/// Stable lowercase identifier, e.g. `fajr`.
#[uniffi::export]
pub fn prayer_id(prayer: Prayer) -> String {
    prayer.id().to_owned()
}

/// Display name of a timetable entry (`Jumu'a` for Friday's Dhuhr).
#[uniffi::export]
pub fn prayer_time_display_name(time: PrayerTime) -> String {
    time.display_name().to_owned()
}

/// Arabic display name of a timetable entry.
#[uniffi::export]
pub fn prayer_time_display_arabic_name(time: PrayerTime) -> String {
    time.display_arabic_name().to_owned()
}

/// Stable identifier for a notification event.
#[uniffi::export]
pub fn prayer_event_id(event: PrayerEvent) -> String {
    event.id()
}

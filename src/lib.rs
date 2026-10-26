//! # MawaKit
//!
//! Prayer-time logic for mosque timetables, shared by apps on every platform.
//!
//! The crate is pure: it never touches the network, the clock or the disk. Apps pass
//! in a mosque's configuration and the current time, and get back:
//!
//! - the day's schedule ([`MosqueTimetable::schedule_for`]), with iqama times and
//!   Jumu'a on Fridays,
//! - what to count down to right now ([`MosqueTimetable::status_at`]),
//! - when a timetable view next changes ([`MosqueTimetable::next_boundary`]), which is
//!   exactly what widget timelines need,
//! - the adhan and iqama moments to notify about ([`MosqueTimetable::events_after`]),
//! - countdown text and how long it stays valid ([`countdown`]).
//!
//! Timetables come from [Mawaqit](https://mawaqit.net) through the [`mawaqit`] adapter.
//!
//! ```
//! use mawakit::{mawaqit, MosqueTimetable};
//! # let html = include_str!("../spec/fixtures/assunna-hannover.html");
//! let configuration = mawaqit::MosqueConfiguration::from_page(html)?;
//! let fetched_at: jiff::Timestamp = "2026-10-23T08:00:00Z".parse()?;
//! let timetable = MosqueTimetable::new(configuration, fetched_at);
//!
//! let now: jiff::Timestamp = "2026-10-23T12:00:00Z".parse()?;
//! let status = timetable.status_at(now).expect("a next prayer");
//! assert_eq!(status.prayer.prayer, mawakit::Prayer::Asr);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod countdown;
mod error;
pub mod mawaqit;
mod prayer;
mod schedule;

#[cfg(feature = "uniffi")]
mod ffi;

pub use error::{MawaKitError, MawaKitError as Error};
pub use prayer::Prayer;
pub use schedule::{
    DaySchedule, MosqueTimetable, PrayerEvent, PrayerEventKind, PrayerPhase, PrayerStatus,
    PrayerTime,
};

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

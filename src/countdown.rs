//! Countdown text, and how long it stays valid so displays can sleep instead of poll.

use jiff::Timestamp;

use crate::PrayerStatus;

/// Short form for status bars: `1h 23m`, `23m`, `45s`.
///
/// Minutes are rounded up so the countdown never shows `0m` before the time is
/// reached, matching how Mawaqit's own displays count down. Between 59 and 60
/// seconds it shows `1m` rather than `60s`.
pub fn compact(seconds: f64) -> String {
    let seconds = seconds.max(0.0);
    if seconds <= 59.0 {
        return format!("{}s", seconds.ceil() as i64);
    }
    let total_minutes = (seconds / 60.0).ceil() as i64;
    match (total_minutes / 60, total_minutes % 60) {
        (0, minutes) => format!("{minutes}m"),
        (hours, 0) => format!("{hours}h"),
        (hours, minutes) => format!("{hours}h {minutes}m"),
    }
}

/// Clock form for panels: `1:23:45` or `23:45`.
pub fn clock(seconds: f64) -> String {
    let total = seconds.max(0.0).ceil() as i64;
    let (hours, minutes, secs) = (total / 3600, (total % 3600) / 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{secs:02}")
    } else {
        format!("{minutes}:{secs:02}")
    }
}

/// How long until [`compact`] shows a different string for a countdown that has
/// `remaining` seconds left.
pub fn time_until_compact_change(remaining: f64) -> f64 {
    if remaining <= 0.0 {
        return 0.0;
    }
    if remaining <= 59.0 {
        // Seconds are rounded up: "5s" shows until exactly 4 seconds remain.
        return remaining - (remaining.ceil() - 1.0);
    }
    // Minutes are rounded up: "3m" shows until exactly 2 minutes remain, and "1m"
    // until the 59-second mark, where the seconds display takes over.
    let minutes = (remaining / 60.0).ceil();
    remaining - ((minutes - 1.0) * 60.0).max(59.0)
}

/// How long a status bar label for `status` stays valid.
///
/// - `shows_countdown`: whether the label shows the time left (`1h 23m`) or only the
///   prayer and its time, which change only when the status does.
/// - `max_interval`: upper bound, so periodic work (refresh checks) still runs.
pub fn refresh_interval(
    status: Option<&PrayerStatus>,
    now: Timestamp,
    shows_countdown: bool,
    max_interval: f64,
) -> f64 {
    let Some(status) = status else {
        return max_interval;
    };
    let remaining = status.remaining(now);
    let valid_for = if shows_countdown {
        time_until_compact_change(remaining)
    } else {
        remaining
    };
    // Land just after the change rather than just before it.
    (valid_for + 0.02).max(0.05).min(max_interval)
}

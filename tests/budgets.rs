//! Work budgets: simulate a full day and count how often a status bar clock wakes up
//! and how often a timetable view redraws. These depend only on the logic, not on the
//! machine, so tight limits are safe; a regression such as ticking every second fails.

use jiff::Timestamp;
use mawakit::{MosqueTimetable, PrayerEventKind, countdown, mawaqit};

const FIXTURE: &str = include_str!("../spec/fixtures/assunna-hannover.html");

/// Friday 23 October 2026 in Hannover: Jumu'a, four iqamas, Isha rolling over to Fajr.
fn day() -> (MosqueTimetable, Timestamp, Timestamp) {
    let configuration =
        mawaqit::MosqueConfiguration::from_page(FIXTURE).unwrap_or_else(|e| unreachable!("{e}"));
    let parse = |s: &str| {
        s.parse::<Timestamp>()
            .unwrap_or_else(|e| unreachable!("{e}"))
    };
    let timetable = MosqueTimetable::new(configuration, parse("2026-10-23T08:00:00Z"));
    (
        timetable,
        parse("2026-10-22T22:00:00Z"),
        parse("2026-10-23T22:00:00Z"),
    )
}

fn label(timetable: &MosqueTimetable, now: Timestamp, shows_countdown: bool) -> String {
    match timetable.status_at(now) {
        None => "-".into(),
        Some(s) if shows_countdown => format!(
            "{:?} {:?} {} {}",
            s.phase,
            s.prayer.prayer,
            s.target,
            countdown::compact(s.remaining(now))
        ),
        Some(s) => format!("{:?} {:?} {}", s.phase, s.prayer.prayer, s.target),
    }
}

/// Wake-ups over the day, and how many found the label unchanged (wasted).
fn simulate_status_bar(shows_countdown: bool) -> (usize, usize) {
    let (timetable, start, end) = day();
    let (mut now, mut wakeups, mut wasted) = (start, 0, 0);
    let mut shown = label(&timetable, now, shows_countdown);
    while now < end {
        let status = timetable.status_at(now);
        let sleep = countdown::refresh_interval(status.as_ref(), now, shows_countdown, 900.0);
        now = now
            .checked_add(jiff::SignedDuration::from_secs_f64(sleep))
            .unwrap_or_else(|e| unreachable!("{e}"));
        wakeups += 1;
        let next = label(&timetable, now, shows_countdown);
        if next == shown {
            wasted += 1;
        }
        shown = next;
    }
    (wakeups, wasted)
}

#[test]
fn countdown_label_wakes_only_when_its_text_changes() {
    let (wakeups, wasted) = simulate_status_bar(true);
    println!("status bar wake-ups per day (countdown): {wakeups}, wasted: {wasted}");
    // Once a minute, plus once a second in the final minute before each of the
    // day's 9 countdown targets: ≈ 1440 + 9 × 59.
    assert!(wakeups <= 2_000, "woke {wakeups} times in a day");
    assert_eq!(wasted, 0, "{wasted} wake-ups found the label unchanged");
}

#[test]
fn plain_label_wakes_only_at_status_changes() {
    let (wakeups, _) = simulate_status_bar(false);
    println!("status bar wake-ups per day (no countdown): {wakeups}");
    // ~9 status changes plus at most 96 fifteen-minute safety ticks.
    assert!(wakeups <= 9 + 96, "woke {wakeups} times in a day");
}

#[test]
fn timetable_redraws_only_at_prayer_boundaries() {
    let (timetable, start, end) = day();
    let (mut now, mut redraws) = (start, 0);
    while now < end {
        now = timetable.next_boundary(now);
        redraws += 1;
    }
    println!("timetable redraws per day: {redraws}");
    // 6 adhans/sunrise + 4 iqamas + midnight.
    assert!(redraws <= 11, "redrew {redraws} times in a day");
}

#[test]
fn notification_window_fits_platform_limits() {
    let (timetable, start, _) = day();
    let events = timetable.events_after(
        start,
        2,
        vec![PrayerEventKind::Adhan, PrayerEventKind::Iqama],
    );
    // macOS and iOS keep at most 64 pending notifications per app.
    assert!(
        events.len() <= 60,
        "{} notifications would be scheduled",
        events.len()
    );
}

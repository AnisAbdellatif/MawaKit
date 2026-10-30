//! Runs the shared spec (`spec/cases.json`): golden cases that every MawaKit
//! implementation and binding must reproduce exactly.

use jiff::Timestamp;
use mawakit::{
    DaySchedule, Error, MosqueTimetable, Prayer, PrayerEventKind, PrayerPhase, PrayerStatus,
    PrayerTime, countdown, mawaqit,
};
use serde_json::{Map, Value, json};

const FIXTURE: &str = include_str!("../spec/fixtures/assunna-hannover.html");
const CASES: &str = include_str!("../spec/cases.json");

/// Collects mismatches so one run reports all of them, with context.
#[derive(Default)]
struct Report {
    failures: Vec<String>,
    checked: usize,
}

impl Report {
    fn check(&mut self, context: impl FnOnce() -> String, expected: &Value, actual: &Value) {
        self.checked += 1;
        if expected != actual {
            self.failures.push(format!(
                "{}\n  expected: {expected}\n  actual:   {actual}",
                context()
            ));
        }
    }

    fn check_float(&mut self, context: impl FnOnce() -> String, expected: &Value, actual: f64) {
        self.checked += 1;
        let ok = expected.as_f64().is_some_and(|e| (e - actual).abs() < 1e-9);
        if !ok {
            self.failures.push(format!(
                "{}\n  expected: {expected}\n  actual:   {actual}",
                context()
            ));
        }
    }

    fn finish(self) {
        let shown: Vec<_> = self.failures.iter().take(25).cloned().collect();
        assert!(
            self.failures.is_empty(),
            "{} of {} spec checks failed (first {} shown):\n\n{}",
            self.failures.len(),
            self.checked,
            shown.len(),
            shown.join("\n\n")
        );
        println!("{} spec checks passed", self.checked);
    }
}

fn cases() -> Value {
    serde_json::from_str(CASES).unwrap_or_else(|e| unreachable!("spec/cases.json is invalid: {e}"))
}

fn at(value: &Value) -> Timestamp {
    let seconds = value
        .as_i64()
        .unwrap_or_else(|| unreachable!("timestamp expected, got {value}"));
    Timestamp::from_second(seconds).unwrap_or_else(|e| unreachable!("{e}"))
}

fn ts(t: Timestamp) -> Value {
    assert_eq!(t.subsec_nanosecond(), 0, "non-integral timestamp {t}");
    json!(t.as_second())
}

fn time_json(t: &PrayerTime) -> Value {
    json!({ "prayer": t.prayer.id(), "adhan": ts(t.adhan), "iqama": t.iqama.map(ts), "isJumua": t.is_jumua })
}

fn schedule_json(schedule: Option<&DaySchedule>) -> Value {
    schedule.map_or(Value::Null, |s| {
        json!({
            "day": ts(s.day),
            "times": s.times.iter().map(time_json).collect::<Vec<_>>(),
            "additionalJumua": s.additional_jumua.iter().copied().map(ts).collect::<Vec<_>>(),
        })
    })
}

fn status_json(status: Option<&PrayerStatus>) -> Value {
    status.map_or(Value::Null, |s| {
        json!({
            "phase": match s.phase { PrayerPhase::Adhan => "adhan", PrayerPhase::Iqama => "iqama" },
            "prayer": time_json(&s.prayer),
            "target": ts(s.target),
            "intervalStart": ts(s.interval_start),
        })
    })
}

fn timetable(case: &Value) -> MosqueTimetable {
    let base = mawaqit::extract_conf_data(FIXTURE).unwrap_or_else(|e| unreachable!("{e}"));
    let mut object: Map<String, Value> =
        serde_json::from_str(base).unwrap_or_else(|e| unreachable!("{e}"));
    if let Some(patch) = case["patch"].as_object() {
        for (key, value) in patch {
            object.insert(key.clone(), value.clone());
        }
    }
    MosqueTimetable::from_configuration_json(
        &Value::Object(object).to_string(),
        at(&case["fetchedAt"]),
    )
    .unwrap_or_else(|e| unreachable!("{e}"))
}

#[test]
fn timetables() {
    let cases = cases();
    let mut report = Report::default();

    for case in cases["timetables"].as_array().into_iter().flatten() {
        let id = case["id"].as_str().unwrap_or("?");
        let timetable = timetable(case);

        for entry in case["schedules"].as_array().into_iter().flatten() {
            let date = at(&entry["date"]);
            let actual = schedule_json(timetable.schedule_for(date).as_ref());
            report.check(
                || format!("[{id}] schedule_for({date})"),
                &entry["schedule"],
                &actual,
            );
        }

        for entry in case["statuses"].as_array().into_iter().flatten() {
            let now = at(&entry["at"]);
            let status = timetable.status_at(now);
            report.check(
                || format!("[{id}] status_at({now})"),
                &entry["status"],
                &status_json(status.as_ref()),
            );

            let displayed = timetable
                .displayed_schedule(now)
                .map(|s| ts(s.day))
                .unwrap_or(Value::Null);
            report.check(
                || format!("[{id}] displayed_schedule({now})"),
                &entry["displayedDay"],
                &displayed,
            );

            let refresh = countdown::refresh_interval(status.as_ref(), now, true, 900.0);
            report.check_float(
                || format!("[{id}] refresh_interval countdown ({now})"),
                &entry["refreshCountdown"],
                refresh,
            );
            let refresh = countdown::refresh_interval(status.as_ref(), now, false, 900.0);
            report.check_float(
                || format!("[{id}] refresh_interval plain ({now})"),
                &entry["refreshPlain"],
                refresh,
            );
        }

        for entry in case["boundaries"].as_array().into_iter().flatten() {
            let after = at(&entry["after"]);
            report.check(
                || format!("[{id}] next_boundary({after})"),
                &entry["next"],
                &ts(timetable.next_boundary(after)),
            );
        }

        for entry in case["events"].as_array().into_iter().flatten() {
            let after = at(&entry["after"]);
            let days = entry["days"]
                .as_u64()
                .and_then(|d| u32::try_from(d).ok())
                .unwrap_or(0);
            let kinds: Vec<PrayerEventKind> = entry["kinds"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|k| match k.as_str() {
                    Some("adhan") => Some(PrayerEventKind::Adhan),
                    Some("iqama") => Some(PrayerEventKind::Iqama),
                    _ => None,
                })
                .collect();
            let actual: Vec<Value> = timetable
                .events_after(after, days, kinds)
                .iter()
                .map(|e| json!({ "kind": e.kind.id(), "prayer": time_json(&e.prayer), "date": ts(e.date) }))
                .collect();
            report.check(
                || format!("[{id}] events_after({after}, {days})"),
                &entry["events"],
                &Value::Array(actual),
            );
        }
    }
    report.finish();
}

#[test]
fn countdown_text() {
    let mut report = Report::default();
    for entry in cases()["countdown"].as_array().into_iter().flatten() {
        let seconds = entry["seconds"].as_f64().unwrap_or(f64::NAN);
        report.check(
            || format!("compact({seconds})"),
            &entry["compact"],
            &json!(countdown::compact(seconds)),
        );
        report.check(
            || format!("clock({seconds})"),
            &entry["clock"],
            &json!(countdown::clock(seconds)),
        );
        report.check_float(
            || format!("time_until_compact_change({seconds})"),
            &entry["untilChange"],
            countdown::time_until_compact_change(seconds),
        );
    }
    report.finish();
}

#[test]
fn mosque_links() {
    let mut report = Report::default();
    for entry in cases()["links"].as_array().into_iter().flatten() {
        let input = entry["input"].as_str().unwrap_or_default();
        report.check(
            || format!("mosque_slug({input:?})"),
            &entry["slug"],
            &json!(mawaqit::mosque_slug(input)),
        );
    }
    report.finish();
}

#[test]
fn page_extraction() {
    let mut report = Report::default();
    for entry in cases()["extraction"].as_array().into_iter().flatten() {
        let html = entry["html"].as_str().unwrap_or_default();
        let actual = match mawaqit::MosqueConfiguration::from_page(html) {
            Ok(configuration) => json!({ "name": configuration.name }),
            Err(Error::ConfDataNotFound) => json!({ "error": "notFound" }),
            Err(Error::ConfDataUnterminated) => json!({ "error": "unterminated" }),
            Err(Error::InvalidConfiguration { .. }) => json!({ "error": "invalid" }),
            Err(Error::InvalidSearchResponse { .. }) => unreachable!("from_page never searches"),
        };
        report.check(|| format!("from_page({html:?})"), &entry["result"], &actual);
    }
    report.finish();
}

#[test]
fn configuration_parsing() {
    let mut report = Report::default();
    for entry in cases()["configuration"].as_array().into_iter().flatten() {
        let input = entry["json"].as_str().unwrap_or_default();
        let actual = match mawaqit::MosqueConfiguration::from_json(input) {
            Ok(c) => json!({
                "name": c.name,
                "displayName": c.display_name(),
                "timezone": c.timezone,
                "iqamaEnabled": c.iqama_enabled,
                "jumua": c.jumua,
                "jumuaAsDuhr": c.jumua_as_duhr,
                "hijriAdjustment": c.hijri_adjustment,
            }),
            Err(e) => json!({ "error": e.to_string() }),
        };
        report.check(|| format!("from_json({input})"), &entry["decoded"], &actual);
    }
    report.finish();
}

#[test]
fn prayer_names() {
    let names = &cases()["names"];
    let mut report = Report::default();
    for prayer in Prayer::ALL {
        let actual = json!({ "name": prayer.name(), "arabic": prayer.arabic_name() });
        report.check(
            || format!("names of {prayer:?}"),
            &names[prayer.id()],
            &actual,
        );
    }
    let jumua = PrayerTime {
        prayer: Prayer::Dhuhr,
        adhan: Timestamp::UNIX_EPOCH,
        iqama: None,
        is_jumua: true,
    };
    report.check(
        || "jumua names".into(),
        &names["jumua"],
        &json!({ "name": jumua.display_name(), "arabic": jumua.display_arabic_name() }),
    );
    report.finish();
}

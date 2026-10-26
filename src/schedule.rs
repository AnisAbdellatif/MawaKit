use jiff::civil::{Date, Weekday};
use jiff::tz::TimeZone;
use jiff::{Timestamp, ToSpan, Zoned};

use crate::mawaqit::MosqueConfiguration;
use crate::prayer::{JUMUA_ARABIC_NAME, JUMUA_NAME};
use crate::{Error, Prayer};

/// One entry of a day's timetable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PrayerTime {
    /// Which prayer (or sunrise).
    pub prayer: Prayer,
    /// When it's due.
    pub adhan: Timestamp,
    /// When the congregation starts. `None` for sunrise, Jumu'a, and mosques
    /// without iqama times.
    pub iqama: Option<Timestamp>,
    /// True when this is Friday's Dhuhr replaced by Jumu'a.
    pub is_jumua: bool,
}

impl PrayerTime {
    /// When the prayer starts in congregation: the iqama, else the adhan.
    pub fn start(&self) -> Timestamp {
        self.iqama.unwrap_or(self.adhan)
    }

    /// Transliterated name, `"Jumu'a"` for Friday's Dhuhr.
    pub fn display_name(&self) -> &'static str {
        if self.is_jumua {
            JUMUA_NAME
        } else {
            self.prayer.name()
        }
    }

    /// Arabic name, `"الجمعة"` for Friday's Dhuhr.
    pub fn display_arabic_name(&self) -> &'static str {
        if self.is_jumua {
            JUMUA_ARABIC_NAME
        } else {
            self.prayer.arabic_name()
        }
    }
}

/// A day's timetable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct DaySchedule {
    /// Midnight of the day, in the mosque's time zone.
    pub day: Timestamp,
    /// All six entries (including sunrise) in chronological order.
    pub times: Vec<PrayerTime>,
    /// Further Jumu'a sessions on Fridays, if the mosque holds several.
    pub additional_jumua: Vec<Timestamp>,
}

impl DaySchedule {
    /// The five obligatory prayers (everything but sunrise).
    pub fn prayers(&self) -> impl Iterator<Item = &PrayerTime> {
        self.times.iter().filter(|t| t.prayer.is_obligatory())
    }

    /// The entry for `prayer`, if the timetable has it.
    pub fn time(&self, prayer: Prayer) -> Option<&PrayerTime> {
        self.times.iter().find(|t| t.prayer == prayer)
    }
}

/// What is being counted down to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum PrayerPhase {
    /// Waiting for the adhan.
    Adhan,
    /// The adhan has been called; waiting for the iqama.
    Iqama,
}

/// The countdown to show at a given moment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PrayerStatus {
    /// Adhan or iqama.
    pub phase: PrayerPhase,
    /// The prayer concerned.
    pub prayer: PrayerTime,
    /// The moment being counted down to.
    pub target: Timestamp,
    /// When the current countdown started (previous prayer, or the adhan).
    pub interval_start: Timestamp,
}

impl PrayerStatus {
    /// Seconds until the target, never negative.
    pub fn remaining(&self, now: Timestamp) -> f64 {
        seconds_between(now, self.target).max(0.0)
    }

    /// Fraction of the current interval that has elapsed, from 0 to 1.
    pub fn progress(&self, now: Timestamp) -> f64 {
        let total = seconds_between(self.interval_start, self.target);
        if total <= 0.0 {
            return 1.0;
        }
        (seconds_between(self.interval_start, now) / total).clamp(0.0, 1.0)
    }
}

/// Kind of moment to notify about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum PrayerEventKind {
    /// The adhan is due.
    Adhan,
    /// The iqama is due.
    Iqama,
}

impl PrayerEventKind {
    /// Stable lowercase identifier.
    pub fn id(self) -> &'static str {
        match self {
            PrayerEventKind::Adhan => "adhan",
            PrayerEventKind::Iqama => "iqama",
        }
    }
}

/// A moment worth notifying about: an adhan or an iqama.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PrayerEvent {
    /// Adhan or iqama.
    pub kind: PrayerEventKind,
    /// The prayer concerned.
    pub prayer: PrayerTime,
    /// When it happens.
    pub date: Timestamp,
}

impl PrayerEvent {
    /// Stable across recomputations, so it can serve as a notification identifier.
    pub fn id(&self) -> String {
        format!(
            "{}.{}.{}",
            self.kind.id(),
            self.prayer.prayer.id(),
            self.date.as_second()
        )
    }
}

/// A mosque's timetable, turned into concrete moments in time.
///
/// Calendar times are local wall-clock times, read in the mosque's time zone, so
/// daylight-saving changes are handled. On the day the configuration was fetched,
/// Mawaqit's own `times` take precedence because they include manual adjustments.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct MosqueTimetable {
    configuration: MosqueConfiguration,
    time_zone: TimeZone,
    fetched_at: Timestamp,
}

impl MosqueTimetable {
    /// A timetable for `configuration`, fetched at `fetched_at`. Falls back to the
    /// system time zone if the configuration names an unknown one.
    pub fn new(configuration: MosqueConfiguration, fetched_at: Timestamp) -> Self {
        let time_zone = configuration
            .timezone
            .as_deref()
            .and_then(|name| TimeZone::get(name).ok())
            .unwrap_or_else(TimeZone::system);
        Self {
            configuration,
            time_zone,
            fetched_at,
        }
    }

    /// The configuration this timetable was built from.
    pub fn configuration(&self) -> &MosqueConfiguration {
        &self.configuration
    }

    /// The mosque's time zone.
    pub fn time_zone(&self) -> &TimeZone {
        &self.time_zone
    }

    fn zoned(&self, at: Timestamp) -> Zoned {
        at.to_zoned(self.time_zone.clone())
    }

    /// `at` moved by whole calendar days in the mosque's time zone.
    fn add_days(&self, at: Timestamp, days: i64) -> Option<Timestamp> {
        self.zoned(at)
            .checked_add(days.days())
            .ok()
            .map(|z| z.timestamp())
    }

    fn start_of_day(&self, at: Timestamp) -> Option<Zoned> {
        self.zoned(at).start_of_day().ok()
    }
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
impl MosqueTimetable {
    /// The timetable of the day containing `date`, in the mosque's time zone.
    pub fn schedule_for(&self, date: Timestamp) -> Option<DaySchedule> {
        let day = self.start_of_day(date)?;
        let civil = day.date();
        let mut raw = entry(&self.configuration.calendar, civil)?.clone();
        if raw.len() < 6 {
            return None;
        }

        if self.zoned(self.fetched_at).date() == civil {
            if let Some(today) = self.configuration.times.as_ref().filter(|t| t.len() >= 5) {
                let shuruq = self
                    .configuration
                    .shuruq
                    .clone()
                    .unwrap_or_else(|| raw[1].clone());
                raw = vec![
                    today[0].clone(),
                    shuruq,
                    today[1].clone(),
                    today[2].clone(),
                    today[3].clone(),
                    today[4].clone(),
                ];
            }
        }

        let iqama_rules = entry(&self.configuration.iqama_calendar, civil);
        let is_friday = civil.weekday() == Weekday::Friday;

        let mut times = Vec::with_capacity(6);
        for (prayer, text) in Prayer::ALL.into_iter().zip(&raw) {
            let Some(adhan) = self.time(text, &day) else {
                continue;
            };

            if prayer == Prayer::Dhuhr && is_friday {
                if let Some(jumua) = self.jumua_time(adhan, &day) {
                    times.push(PrayerTime {
                        prayer,
                        adhan: jumua,
                        iqama: None,
                        is_jumua: true,
                    });
                    continue;
                }
            }

            let iqama = if self.configuration.iqama_enabled {
                prayer
                    .obligatory_index()
                    .and_then(|index| iqama_rules?.get(index))
                    .and_then(|rule| self.iqama_time(rule, adhan, &day))
            } else {
                None
            };
            times.push(PrayerTime {
                prayer,
                adhan,
                iqama,
                is_jumua: false,
            });
        }
        times.sort_by_key(|t| t.adhan);

        let additional_jumua = if is_friday {
            [&self.configuration.jumua2, &self.configuration.jumua3]
                .into_iter()
                .flatten()
                .filter_map(|text| self.time(text, &day))
                .collect()
        } else {
            Vec::new()
        };

        Some(DaySchedule {
            day: day.timestamp(),
            times,
            additional_jumua,
        })
    }

    /// What to count down to at `now`: the iqama while it is pending, otherwise the
    /// next adhan. Sunrise is never counted down to.
    pub fn status_at(&self, now: Timestamp) -> Option<PrayerStatus> {
        let mut prayers: Vec<PrayerTime> = (-1..=1)
            .filter_map(|offset| self.add_days(now, offset))
            .filter_map(|day| self.schedule_for(day))
            .flat_map(|schedule| {
                schedule
                    .times
                    .into_iter()
                    .filter(|t| t.prayer.is_obligatory())
            })
            .collect();
        prayers.sort_by_key(|t| t.adhan);

        for (index, prayer) in prayers.iter().enumerate() {
            if let Some(iqama) = prayer.iqama {
                if prayer.adhan <= now && now < iqama {
                    return Some(PrayerStatus {
                        phase: PrayerPhase::Iqama,
                        prayer: prayer.clone(),
                        target: iqama,
                        interval_start: prayer.adhan,
                    });
                }
            }
            if prayer.adhan > now {
                let previous_start = index
                    .checked_sub(1)
                    .and_then(|i| prayers.get(i))
                    .map_or(now, PrayerTime::start);
                return Some(PrayerStatus {
                    phase: PrayerPhase::Adhan,
                    prayer: prayer.clone(),
                    target: prayer.adhan,
                    interval_start: previous_start.min(now),
                });
            }
        }
        None
    }

    /// The timetable worth showing at `now`: the day of the prayer being counted
    /// down to. After the day's last prayer that is tomorrow.
    pub fn displayed_schedule(&self, now: Timestamp) -> Option<DaySchedule> {
        let day = self
            .status_at(now)
            .map_or(now, |status| status.prayer.adhan);
        self.schedule_for(day)
    }

    /// The next moment after `after` at which a timetable view looks different: an
    /// adhan or iqama, sunrise, or midnight. Nothing in between needs redrawing.
    pub fn next_boundary(&self, after: Timestamp) -> Timestamp {
        let Some(day) = self.start_of_day(after) else {
            return after.checked_add(1.hour()).unwrap_or(after);
        };
        let midnight = day
            .checked_add(1.day())
            .map(|z| z.timestamp())
            .unwrap_or_else(|_| after.checked_add(24.hours()).unwrap_or(after));

        // Yesterday too: a late Isha iqama can fall after midnight.
        [-1, 0]
            .into_iter()
            .filter_map(|offset| self.add_days(day.timestamp(), offset))
            .filter_map(|d| self.schedule_for(d))
            .flat_map(|schedule| schedule.times)
            .flat_map(|t| [Some(t.adhan), t.iqama])
            .flatten()
            .filter(|&moment| moment > after)
            .min()
            .map_or(midnight, |moment| moment.min(midnight))
    }

    /// Adhan and/or iqama moments after `now`, covering today and the next `days`
    /// days, in chronological order. Sunrise is never included.
    pub fn events_after(
        &self,
        now: Timestamp,
        days: u32,
        kinds: Vec<PrayerEventKind>,
    ) -> Vec<PrayerEvent> {
        if kinds.is_empty() {
            return Vec::new();
        }
        let mut events: Vec<PrayerEvent> = (0..=i64::from(days))
            .filter_map(|offset| self.add_days(now, offset))
            .filter_map(|day| self.schedule_for(day))
            .flat_map(|schedule| {
                schedule
                    .times
                    .into_iter()
                    .filter(|t| t.prayer.is_obligatory())
            })
            .flat_map(|prayer| {
                let adhan = kinds
                    .contains(&PrayerEventKind::Adhan)
                    .then(|| PrayerEvent {
                        kind: PrayerEventKind::Adhan,
                        date: prayer.adhan,
                        prayer: prayer.clone(),
                    });
                let iqama = prayer
                    .iqama
                    .filter(|_| kinds.contains(&PrayerEventKind::Iqama))
                    .map(|date| PrayerEvent {
                        kind: PrayerEventKind::Iqama,
                        date,
                        prayer: prayer.clone(),
                    });
                [adhan, iqama].into_iter().flatten()
            })
            .filter(|event| event.date > now)
            .collect();
        events.sort_by_key(|e| e.date);
        events
    }
}

impl MosqueTimetable {
    /// Parses `HH:mm` as a wall-clock time on `day` in the mosque's time zone.
    /// Hours up to 47 are accepted: some timetables write times after midnight as 24:xx.
    fn time(&self, text: &str, day: &Zoned) -> Option<Timestamp> {
        let mut parts = text
            .trim_matches([' ', '\t'])
            .split(':')
            .filter(|p| !p.is_empty());
        let (hour, minute) = (parts.next()?, parts.next()?);
        if parts.next().is_some() {
            return None;
        }
        let hour: i8 = hour.parse().ok().filter(|h| (0..48).contains(h))?;
        let minute: i8 = minute.parse().ok().filter(|m| (0..60).contains(m))?;

        let date = day.date();
        let zoned = date
            .at(hour % 24, minute, 0, 0)
            .to_zoned(self.time_zone.clone())
            .ok()?;
        let zoned = if hour >= 24 {
            zoned.checked_add(1.day()).ok()?
        } else {
            zoned
        };
        Some(zoned.timestamp())
    }

    /// An iqama rule is either `+N` minutes after the adhan or an absolute `HH:mm`.
    fn iqama_time(&self, rule: &str, adhan: Timestamp, day: &Zoned) -> Option<Timestamp> {
        let rule = rule.trim_matches([' ', '\t']);
        if let Some(offset) = rule.strip_prefix('+') {
            let minutes: i64 = offset.parse().ok().filter(|&m| m > 0)?;
            return adhan.checked_add((minutes * 60).seconds()).ok();
        }
        self.time(rule, day).filter(|&fixed| fixed > adhan)
    }

    fn jumua_time(&self, dhuhr: Timestamp, day: &Zoned) -> Option<Timestamp> {
        if self.configuration.jumua_as_duhr {
            return Some(dhuhr);
        }
        self.configuration
            .jumua
            .as_deref()
            .and_then(|text| self.time(text, day))
    }
}

/// Builds a timetable from a `confData` JSON object (used by the bindings, where
/// the configuration crosses the language boundary as JSON).
impl MosqueTimetable {
    /// Parses `configuration_json` and builds a timetable from it.
    pub fn from_configuration_json(
        configuration_json: &str,
        fetched_at: Timestamp,
    ) -> Result<Self, Error> {
        Ok(Self::new(
            MosqueConfiguration::from_json(configuration_json)?,
            fetched_at,
        ))
    }
}

fn entry(months: &[crate::mawaqit::Month], date: Date) -> Option<&Vec<String>> {
    let month = usize::try_from(date.month()).ok()?.checked_sub(1)?;
    months.get(month)?.get(&date.day().to_string())
}

fn seconds_between(from: Timestamp, to: Timestamp) -> f64 {
    (to.as_nanosecond() - from.as_nanosecond()) as f64 / 1_000_000_000.0
}

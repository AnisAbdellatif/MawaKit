use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::Error;

/// One month of a calendar: day number (`"1"`…`"31"`) to the day's times.
pub type Month = HashMap<String, Vec<String>>;

/// The subset of a mosque's Mawaqit configuration (`confData`) that timetables need.
///
/// Parsing is deliberately lenient: Mawaqit omits or nulls many fields depending on
/// how a mosque is set up, so only `name` and `calendar` are required, and optional
/// fields with an unexpected type are treated as missing.
#[derive(Debug, Clone, PartialEq)]
pub struct MosqueConfiguration {
    /// The mosque's name.
    pub name: String,
    /// Display name, if set.
    pub label: Option<String>,
    /// IANA time zone, e.g. `Europe/Berlin`.
    pub timezone: Option<String>,
    /// The mosque's Mawaqit URL.
    pub url: Option<String>,
    /// Whether the mosque publishes iqama times.
    pub iqama_enabled: bool,
    /// Jumu'a time (`HH:mm`).
    pub jumua: Option<String>,
    /// Second Jumu'a session, if any.
    pub jumua2: Option<String>,
    /// Third Jumu'a session, if any.
    pub jumua3: Option<String>,
    /// When true, Jumu'a is held at the regular Dhuhr time.
    pub jumua_as_duhr: bool,
    /// Days to shift the Hijri date by.
    pub hijri_adjustment: i64,
    /// Today's five prayer times as computed by Mawaqit when the page was fetched,
    /// including manual adjustments the calendar may not reflect.
    pub times: Option<Vec<String>>,
    /// Today's sunrise, matching `times`.
    pub shuruq: Option<String>,
    /// Twelve months, each mapping a day to six times: Fajr, Shuruq, Dhuhr, Asr,
    /// Maghrib, Isha, in local wall-clock time.
    pub calendar: Vec<Month>,
    /// Twelve months, each mapping a day to five iqama rules (one per obligatory
    /// prayer): an offset such as `"+10"` or a time such as `"13:00"`.
    pub iqama_calendar: Vec<Month>,
}

impl MosqueConfiguration {
    /// Parses a `confData` JSON object.
    pub fn from_json(json: &str) -> Result<Self, Error> {
        let value: Value = serde_json::from_str(json).map_err(|e| invalid(e.to_string()))?;
        let object = value
            .as_object()
            .ok_or_else(|| invalid("not a JSON object"))?;
        Self::from_object(object)
    }

    /// Extracts and parses the configuration embedded in a Mawaqit mosque page.
    pub fn from_page(html: &str) -> Result<Self, Error> {
        Self::from_json(super::extract_conf_data(html)?)
    }

    fn from_object(object: &Map<String, Value>) -> Result<Self, Error> {
        let name = object
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("missing name"))?
            .to_owned();
        let calendar = object
            .get("calendar")
            .and_then(months)
            .ok_or_else(|| invalid("missing or malformed calendar"))?;

        let string = |key: &str| object.get(key).and_then(Value::as_str).map(str::to_owned);
        let boolean = |key: &str| object.get(key).and_then(Value::as_bool);

        Ok(Self {
            name,
            label: string("label"),
            timezone: string("timezone"),
            url: string("url"),
            iqama_enabled: boolean("iqamaEnabled").unwrap_or(true),
            jumua: string("jumua"),
            jumua2: string("jumua2"),
            jumua3: string("jumua3"),
            jumua_as_duhr: boolean("jumuaAsDuhr").unwrap_or(false),
            hijri_adjustment: object
                .get("hijriAdjustment")
                .and_then(|v| {
                    v.as_i64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })
                .unwrap_or(0),
            times: object.get("times").and_then(strings),
            shuruq: string("shuruq"),
            calendar,
            iqama_calendar: object
                .get("iqamaCalendar")
                .and_then(months)
                .unwrap_or_default(),
        })
    }

    /// The label if set, otherwise the name.
    pub fn display_name(&self) -> &str {
        match self.label.as_deref().map(str::trim) {
            Some(label) if !label.is_empty() => label,
            _ => &self.name,
        }
    }
}

fn invalid(reason: impl Into<String>) -> Error {
    Error::InvalidConfiguration {
        reason: reason.into(),
    }
}

/// An array of strings, or nothing if any element isn't one.
fn strings(value: &Value) -> Option<Vec<String>> {
    value
        .as_array()?
        .iter()
        .map(|v| v.as_str().map(str::to_owned))
        .collect()
}

/// An array of months, or nothing if anything inside is malformed.
fn months(value: &Value) -> Option<Vec<Month>> {
    value
        .as_array()?
        .iter()
        .map(|month| {
            month
                .as_object()?
                .iter()
                .map(|(day, times)| Some((day.clone(), strings(times)?)))
                .collect()
        })
        .collect()
}

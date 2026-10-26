/// The daily prayer times, in chronological order.
///
/// `Sunrise` (Shuruq) is not a prayer, but it is part of every timetable and marks the
/// end of the Fajr window, so it is listed alongside the others.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum Prayer {
    /// Dawn prayer.
    Fajr,
    /// Sunrise (Shuruq): the end of the Fajr window, not a prayer.
    Sunrise,
    /// Midday prayer; replaced by Jumu'a on Fridays.
    Dhuhr,
    /// Afternoon prayer.
    Asr,
    /// Sunset prayer.
    Maghrib,
    /// Night prayer.
    Isha,
}

impl Prayer {
    /// All six entries of a timetable, in order.
    pub const ALL: [Prayer; 6] = [
        Prayer::Fajr,
        Prayer::Sunrise,
        Prayer::Dhuhr,
        Prayer::Asr,
        Prayer::Maghrib,
        Prayer::Isha,
    ];

    /// The five obligatory prayers, i.e. everything except sunrise.
    pub const OBLIGATORY: [Prayer; 5] = [
        Prayer::Fajr,
        Prayer::Dhuhr,
        Prayer::Asr,
        Prayer::Maghrib,
        Prayer::Isha,
    ];

    /// Whether this is one of the five prayers (not sunrise).
    pub fn is_obligatory(self) -> bool {
        self != Prayer::Sunrise
    }

    /// Stable lowercase identifier, e.g. `"fajr"`.
    pub fn id(self) -> &'static str {
        match self {
            Prayer::Fajr => "fajr",
            Prayer::Sunrise => "sunrise",
            Prayer::Dhuhr => "dhuhr",
            Prayer::Asr => "asr",
            Prayer::Maghrib => "maghrib",
            Prayer::Isha => "isha",
        }
    }

    /// Transliterated name, e.g. `"Fajr"`.
    pub fn name(self) -> &'static str {
        match self {
            Prayer::Fajr => "Fajr",
            Prayer::Sunrise => "Shuruq",
            Prayer::Dhuhr => "Dhuhr",
            Prayer::Asr => "Asr",
            Prayer::Maghrib => "Maghrib",
            Prayer::Isha => "Isha",
        }
    }

    /// Arabic name, e.g. `"الفجر"`.
    pub fn arabic_name(self) -> &'static str {
        match self {
            Prayer::Fajr => "الفجر",
            Prayer::Sunrise => "الشروق",
            Prayer::Dhuhr => "الظهر",
            Prayer::Asr => "العصر",
            Prayer::Maghrib => "المغرب",
            Prayer::Isha => "العشاء",
        }
    }

    /// Position in [`Prayer::OBLIGATORY`], which is also the index of its iqama rule.
    pub(crate) fn obligatory_index(self) -> Option<usize> {
        Prayer::OBLIGATORY.iter().position(|&p| p == self)
    }
}

/// Transliterated name of Friday's congregational prayer.
pub const JUMUA_NAME: &str = "Jumu'a";
/// Arabic name of Friday's congregational prayer.
pub const JUMUA_ARABIC_NAME: &str = "الجمعة";

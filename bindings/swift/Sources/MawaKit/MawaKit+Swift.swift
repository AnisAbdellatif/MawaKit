// Hand-written Swift conveniences over the generated bindings (MawaKit.swift).
import Foundation

extension Prayer: CaseIterable, Identifiable {
    /// All six entries of a timetable, in order.
    public static var allCases: [Prayer] { [.fajr, .sunrise, .dhuhr, .asr, .maghrib, .isha] }

    /// The five obligatory prayers, i.e. everything except sunrise.
    public static var obligatory: [Prayer] { [.fajr, .dhuhr, .asr, .maghrib, .isha] }

    /// Stable lowercase identifier, e.g. `fajr`.
    public var id: String { prayerId(prayer: self) }

    public var isObligatory: Bool { self != .sunrise }

    /// Transliterated name, e.g. `Fajr`.
    public var name: String { prayerName(prayer: self) }

    /// Arabic name, e.g. `الفجر`.
    public var arabicName: String { prayerArabicName(prayer: self) }
}

extension PrayerTime: Identifiable {
    public var id: Prayer { prayer }

    /// `Jumu'a` for Friday's Dhuhr, otherwise the prayer's name.
    public var displayName: String { prayerTimeDisplayName(time: self) }

    public var displayArabicName: String { prayerTimeDisplayArabicName(time: self) }

    /// When the prayer starts in congregation: the iqama, else the adhan.
    public var start: Date { iqama ?? adhan }
}

extension DaySchedule {
    /// The five obligatory prayers (everything but sunrise).
    public var prayers: [PrayerTime] { times.filter(\.prayer.isObligatory) }

    public func time(for prayer: Prayer) -> PrayerTime? {
        times.first { $0.prayer == prayer }
    }
}

extension PrayerStatus {
    /// Seconds until the target, never negative.
    public func remaining(at now: Date) -> TimeInterval {
        max(0, target.timeIntervalSince(now))
    }

    /// Fraction of the current interval that has elapsed, from 0 to 1.
    public func progress(at now: Date) -> Double {
        prayerStatusProgress(status: self, now: now)
    }
}

extension PrayerEvent: Identifiable {
    /// Stable across recomputations, so it can serve as a notification identifier.
    public var id: String { prayerEventId(event: self) }
}

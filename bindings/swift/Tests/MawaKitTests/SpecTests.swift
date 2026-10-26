// Runs the shared spec (spec/cases.json) through the Swift bindings, proving they
// reproduce the core's behaviour across the language boundary.
import Foundation
import MawaKit
import Testing

private let repositoryRoot = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent().deletingLastPathComponent()
    .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()

private func load(_ path: String) throws -> Data {
    try Data(contentsOf: repositoryRoot.appending(path: path))
}

private func spec() throws -> [String: Any] {
    try #require(try JSONSerialization.jsonObject(with: load("spec/cases.json")) as? [String: Any])
}

private func date(_ value: Any?) -> Date {
    Date(timeIntervalSince1970: (value as? NSNumber)?.doubleValue ?? .nan)
}

private func json(_ time: PrayerTime) -> NSDictionary {
    [
        "prayer": time.prayer.id,
        "adhan": Int(time.adhan.timeIntervalSince1970),
        "iqama": time.iqama.map { Int($0.timeIntervalSince1970) } as Any? ?? NSNull(),
        "isJumua": time.isJumua,
    ]
}

private func json(_ schedule: DaySchedule?) -> Any {
    guard let schedule else { return NSNull() }
    return [
        "day": Int(schedule.day.timeIntervalSince1970),
        "times": schedule.times.map(json),
        "additionalJumua": schedule.additionalJumua.map { Int($0.timeIntervalSince1970) },
    ] as NSDictionary
}

private func json(_ status: PrayerStatus?) -> Any {
    guard let status else { return NSNull() }
    return [
        "phase": status.phase == .adhan ? "adhan" : "iqama",
        "prayer": json(status.prayer),
        "target": Int(status.target.timeIntervalSince1970),
        "intervalStart": Int(status.intervalStart.timeIntervalSince1970),
    ] as NSDictionary
}

private func same(_ expected: Any?, _ actual: Any) -> Bool {
    switch (expected, actual) {
    case (is NSNull, is NSNull), (nil, is NSNull): true
    case let (e as NSObject, a as NSObject): e.isEqual(a)
    default: false
    }
}

@Suite("Shared spec through the Swift bindings")
struct SpecTests {
    @Test func timetables() throws {
        let spec = try spec()
        let html = try #require(String(data: try load("spec/fixtures/assunna-hannover.html"), encoding: .utf8))
        let base = try #require(try JSONSerialization.jsonObject(with: Data(extractConfData(html: html).utf8)) as? [String: Any])

        var checked = 0
        var failures: [String] = []
        func check(_ context: @autoclosure () -> String, _ expected: Any?, _ actual: Any) {
            checked += 1
            if !same(expected, actual), failures.count < 20 {
                failures.append("\(context())\n  expected: \(expected ?? "nil")\n  actual:   \(actual)")
            }
        }

        for case let entry as [String: Any] in spec["timetables"] as? [Any] ?? [] {
            let id = entry["id"] as? String ?? "?"
            var configuration = base
            for (key, value) in entry["patch"] as? [String: Any] ?? [:] { configuration[key] = value }
            let configurationJSON = try #require(String(data: JSONSerialization.data(withJSONObject: configuration), encoding: .utf8))
            let timetable = try MosqueTimetable.fromConfigurationJson(configurationJson: configurationJSON, fetchedAt: date(entry["fetchedAt"]))

            for case let item as [String: Any] in entry["schedules"] as? [Any] ?? [] {
                check("[\(id)] scheduleFor(\(item["date"]!))", item["schedule"], json(timetable.scheduleFor(date: date(item["date"]))))
            }
            for case let item as [String: Any] in entry["statuses"] as? [Any] ?? [] {
                let now = date(item["at"])
                let status = timetable.statusAt(now: now)
                check("[\(id)] statusAt(\(item["at"]!))", item["status"], json(status))
                let displayed = timetable.displayedSchedule(now: now).map { Int($0.day.timeIntervalSince1970) as Any } ?? NSNull()
                check("[\(id)] displayedSchedule(\(item["at"]!))", item["displayedDay"], displayed)
                let refresh = statusBarRefreshInterval(status: status, now: now, showsCountdown: true, maxInterval: 900)
                check("[\(id)] refresh(\(item["at"]!))", item["refreshCountdown"], NSNumber(value: refresh))
            }
            for case let item as [String: Any] in entry["boundaries"] as? [Any] ?? [] {
                let next = timetable.nextBoundary(after: date(item["after"]))
                check("[\(id)] nextBoundary(\(item["after"]!))", item["next"], Int(next.timeIntervalSince1970))
            }
        }

        #expect(failures.isEmpty, "\(failures.count) of \(checked) checks failed:\n\(failures.joined(separator: "\n"))")
        print("\(checked) spec checks through the Swift bindings")
        #expect(checked > 15_000)
    }

    @Test func statelessCases() throws {
        let spec = try spec()
        for case let item as [String: Any] in spec["countdown"] as? [Any] ?? [] {
            let seconds = (item["seconds"] as? NSNumber)?.doubleValue ?? .nan
            #expect(countdownCompact(seconds: seconds) == item["compact"] as? String)
            #expect(countdownClock(seconds: seconds) == item["clock"] as? String)
        }
        for case let item as [String: Any] in spec["links"] as? [Any] ?? [] {
            #expect(mosqueSlug(input: item["input"] as? String ?? "") == item["slug"] as? String)
        }
    }

    @Test func errorsCrossTheBoundary() {
        #expect(throws: MawaKitError.ConfDataNotFound) { try extractConfData(html: "<html></html>") }
        #expect(throws: MawaKitError.self) {
            try MosqueTimetable.fromConfigurationJson(configurationJson: "{}", fetchedAt: .now)
        }
    }
}

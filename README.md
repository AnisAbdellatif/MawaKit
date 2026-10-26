# MawaKit

Prayer-time logic for mosque timetables, written once in Rust and used from any language.

- **Day schedules:** the six daily times, iqama times from offset (`+10`) or fixed (`13:00`) rules, and Jumu'a on
  Fridays, including extra sessions.
- **Countdown status:** what to count down to right now (the next adhan, or the iqama once the adhan has been
  called), with progress.
- **Redraw boundaries:** the next moment a timetable view looks different. Widget timelines are built from exactly
  this.
- **Notification events:** upcoming adhan and iqama moments.
- **Countdown text:** `1h 23m` / `1:23:45`, plus how long the text stays valid, so displays sleep instead of polling.
- **[Mawaqit](https://mawaqit.net) adapter:** extracts a mosque's configuration from its public page, and turns
  pasted links into mosque slugs.

Calendar times are read in the mosque's time zone (via [`jiff`](https://docs.rs/jiff)), so daylight-saving changes
are handled.

The core is **pure**: no network, clock or disk access. Apps fetch pages, store them and schedule work with their
platform's own APIs, then ask MawaKit what to show.

## Rust

```rust
use mawakit::{mawaqit, MosqueTimetable};

let configuration = mawaqit::MosqueConfiguration::from_page(&html)?;
let timetable = MosqueTimetable::new(configuration, fetched_at);

if let Some(status) = timetable.status_at(now) {
    println!("{} in {}", status.prayer.display_name(), mawakit::countdown::compact(status.remaining(now)));
}
```

## Swift

`make swift` builds a universal (arm64 + x86_64) static library as `bindings/swift/MawaKitFFI.xcframework` and
generates the Swift API with [UniFFI](https://mozilla.github.io/uniffi-rs/). The repository root is a Swift package
(`Package.swift`). Only the Command Line Tools are needed; Xcode isn't.

```swift
import MawaKit

let timetable = try MosqueTimetable.fromConfigurationJson(configurationJson: json, fetchedAt: fetchedAt)
if let status = timetable.statusAt(now: .now) {
    print(status.prayer.displayName, countdownCompact(seconds: status.remaining(at: .now)))
}
```

Kotlin, Python and WebAssembly bindings can be generated the same way when an app needs them.

## The shared spec

`spec/cases.json` holds golden test cases: a captured mosque page plus around 26,000 expected outputs. They cover
schedules for a whole year, both daylight-saving changes (Europe and the US), the year change, configuration variants,
and the exact second before, at and after every adhan and iqama. Every implementation and binding must reproduce them:

- `tests/spec.rs` checks the Rust core (26,471 checks).
- `bindings/swift/Tests` checks the Swift bindings against the same file (19,980 checks).

The cases were recorded from the original Swift implementation of SalatWidget. Changing behaviour means changing
the spec deliberately, in the same commit.

## Development

```bash
make test       # Rust tests (spec + work budgets), then the Swift bindings against the spec
make lint       # rustfmt + clippy (unwrap/expect/panic are denied)
make swift      # build the Swift package's library and generated API
make hooks      # lint + Rust tests before commits, everything before pushes
```

## Status

Version 0.x: the API may still change. The source is public, but the crate isn't on crates.io yet. Before publishing
it there:
- agree the data use (and the name) with Mawaqit,
- settle the API,
- set up CI on macOS, Linux and Windows.

## License

[MIT](LICENSE). Prayer times are provided by MAWAQIT; this project isn't affiliated with MAWAQIT.

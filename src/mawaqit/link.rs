/// Turns what a user pastes into a mosque slug.
///
/// Accepts, for example:
/// - `https://mawaqit.net/de/assunna-hannover`
/// - `mawaqit.net/en/w/assunna-hannover?showOnly5PrayerTimes=0`
/// - `assunna-hannover`
pub fn mosque_slug(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.contains('/') || trimmed.to_lowercase().contains("mawaqit.") {
        let rest = match trimmed.find("://") {
            Some(index) => &trimmed[index + 3..],
            None => trimmed,
        };
        let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
        let (authority, after) = rest.split_at(authority_end);

        // Drop credentials and port: user:pass@host:port.
        let host = authority.rsplit('@').next().unwrap_or(authority);
        let host = host.split(':').next().unwrap_or(host).to_lowercase();
        if host != "mawaqit.net" && !host.ends_with(".mawaqit.net") {
            return None;
        }

        // Paths look like /<lang>/<slug>, /<lang>/w/<slug> or /<lang>/m/<slug>.
        let path_end = after.find(['?', '#']).unwrap_or(after.len());
        let components: Vec<&str> = after[..path_end]
            .split('/')
            .filter(|c| !c.is_empty())
            .collect();
        let last = components.last()?.to_lowercase();
        return (components.len() >= 2 && is_valid_slug(&last)).then_some(last);
    }

    let candidate = trimmed.to_lowercase();
    is_valid_slug(&candidate).then_some(candidate)
}

fn is_valid_slug(value: &str) -> bool {
    (2..=200).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}

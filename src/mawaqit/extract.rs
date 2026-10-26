use crate::Error;

/// Returns the `confData` JSON object embedded in a Mawaqit mosque page.
///
/// Pages contain `var confData = { … };` in an inline script. The object is found by
/// matching braces, skipping braces inside string literals, so the rest of the page
/// doesn't need to be valid JavaScript.
pub fn extract_conf_data(html: &str) -> Result<&str, Error> {
    const MARKER: &[u8] = b"confData";
    let bytes = html.as_bytes();

    let mut search_from = 0;
    while let Some(found) = find(bytes, MARKER, search_from) {
        search_from = found + MARKER.len();
        // Expect optional whitespace, `=`, optional whitespace, then `{`.
        let mut i = skip_whitespace(bytes, search_from);
        if bytes.get(i) != Some(&b'=') {
            continue;
        }
        i = skip_whitespace(bytes, i + 1);
        if bytes.get(i) != Some(&b'{') {
            continue;
        }
        let end = matching_brace(bytes, i).ok_or(Error::ConfDataUnterminated)?;
        // Both ends are ASCII braces, so this slice is valid UTF-8.
        return html.get(i..=end).ok_or(Error::ConfDataUnterminated);
    }
    Err(Error::ConfDataNotFound)
}

fn skip_whitespace(bytes: &[u8], mut i: usize) -> usize {
    while matches!(bytes.get(i), Some(b' ' | b'\t' | b'\n' | b'\r')) {
        i += 1;
    }
    i
}

fn find(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    haystack
        .get(from..)?
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| from + offset)
}

/// Index of the `}` closing the object that opens at `start`.
fn matching_brace(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (i, &byte) in bytes.iter().enumerate().skip(start) {
        if in_string {
            match byte {
                _ if escaped => escaped = false,
                b'\\' => escaped = true,
                b'"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

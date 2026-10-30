use serde_json::Value;

use crate::Error;

const ENDPOINT: &str = "https://mawaqit.net/api/2.0/mosque/search";
const TOKEN_HEADER: &str = "Api-Access-Token";

/// An HTTP header to send with a request.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct HttpHeader {
    /// Header name.
    pub name: String,
    /// Header value.
    pub value: String,
}

/// A mosque-search request for the app to send with its own HTTP client: a `GET` of
/// [`url`](Self::url) with [`headers`](Self::headers). Pass the response body to
/// [`parse_search_results`].
///
/// Mawaqit's search API needs an API access token, which Mawaqit issues on request.
/// Keep it out of source control.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct MosqueSearchRequest {
    /// The full request URL, query included.
    pub url: String,
    /// Headers to send, including the access token.
    pub headers: Vec<HttpHeader>,
}

impl MosqueSearchRequest {
    /// Searches mosques by name or place.
    pub fn by_name(word: &str, token: &str) -> Self {
        Self::new(format!("?word={}", percent_encode(word.trim())), token)
    }

    /// Searches mosques near a location, nearest first.
    pub fn nearby(latitude: f64, longitude: f64, token: &str) -> Self {
        Self::new(format!("?lat={latitude:.6}&lon={longitude:.6}"), token)
    }

    fn new(query: String, token: &str) -> Self {
        let header = |name: &str, value: &str| HttpHeader {
            name: name.to_owned(),
            value: value.to_owned(),
        };
        Self {
            url: format!("{ENDPOINT}{query}"),
            headers: vec![
                header(TOKEN_HEADER, token),
                header("Accept", "application/json"),
            ],
        }
    }
}

/// One mosque found by a search.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct MosqueSearchResult {
    /// Identifies the mosque; see [`page_url`](super::page_url).
    pub slug: String,
    /// Display name.
    pub label: String,
    /// Address, empty when unknown.
    pub place: String,
    /// Distance in meters from the searched location, for nearby searches.
    pub proximity_meters: Option<u32>,
}

/// Parses the body of a search response. Entries without a slug are skipped.
pub fn parse_search_results(body: &str) -> Result<Vec<MosqueSearchResult>, Error> {
    let value: Value = serde_json::from_str(body).map_err(|e| invalid(e.to_string()))?;
    let entries = value
        .as_array()
        .ok_or_else(|| invalid("expected a JSON array".to_owned()))?;

    Ok(entries
        .iter()
        .filter_map(|entry| {
            let slug = text(entry, "slug")?;
            let label = text(entry, "label")
                .or_else(|| text(entry, "name"))
                .unwrap_or(slug);
            Some(MosqueSearchResult {
                slug: slug.to_owned(),
                label: label.to_owned(),
                place: text(entry, "localisation").unwrap_or_default().to_owned(),
                proximity_meters: entry
                    .get("proximity")
                    .and_then(Value::as_f64)
                    .map(|meters| meters.clamp(0.0, f64::from(u32::MAX)) as u32),
            })
        })
        .collect())
}

fn text<'a>(entry: &'a Value, key: &str) -> Option<&'a str> {
    entry
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
}

fn invalid(reason: String) -> Error {
    Error::InvalidSearchResponse { reason }
}

/// Percent-encodes everything but RFC 3986 unreserved characters.
fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_name_request() {
        let request = MosqueSearchRequest::by_name(" Hannover Süd ", "secret");
        assert_eq!(
            request.url,
            "https://mawaqit.net/api/2.0/mosque/search?word=Hannover%20S%C3%BCd"
        );
        assert_eq!(request.headers[0].name, "Api-Access-Token");
        assert_eq!(request.headers[0].value, "secret");
    }

    #[test]
    fn builds_nearby_request() {
        let request = MosqueSearchRequest::nearby(52.37, -9.7, "t");
        assert!(request.url.ends_with("?lat=52.370000&lon=-9.700000"));
    }

    #[test]
    fn parses_results() {
        let body = r#"[
            {"slug":"a-mosque","label":"A Mosque","localisation":"Hannover","proximity":350},
            {"slug":"b-mosque","name":"B","proximity":null},
            {"label":"no slug"}
        ]"#;
        let results = parse_search_results(body).unwrap_or_default();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].proximity_meters, Some(350));
        assert_eq!(results[0].place, "Hannover");
        assert_eq!(results[1].label, "B");
        assert_eq!(results[1].proximity_meters, None);
    }

    #[test]
    fn rejects_non_arrays() {
        assert!(parse_search_results("{}").is_err());
        assert!(parse_search_results("nope").is_err());
    }
}

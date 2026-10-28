//! Valida lo que pega el usuario. Solo se acepta un post de X y se devuelve su ID numérico:
//! el backend nunca hace peticiones a una URL arbitraria (evita SSRF).

use crate::error::fail;
use reqwest::Url;

const HOSTS: [&str; 6] = [
    "x.com", "www.x.com", "mobile.x.com", "twitter.com", "www.twitter.com", "mobile.twitter.com",
];

pub fn status_id(input: &str) -> Result<String, String> {
    let url = Url::parse(input.trim()).map_err(|_| fail("invalid_link"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(fail("link_scheme"));
    }
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    if !HOSTS.contains(&host.as_str()) {
        return Err(fail("host_not_allowed"));
    }
    let segments: Vec<&str> = url.path_segments().map(|s| s.collect()).unwrap_or_default();
    // /usuario/status/ID, /i/status/ID o /i/web/status/ID; lo que venga detrás (/photo/1) se ignora
    let id = segments
        .windows(2)
        .find(|w| w[0] == "status" || w[0] == "statuses")
        .map(|w| w[1])
        .ok_or_else(|| fail("not_a_post"))?;
    if id.is_empty() || id.len() > 20 || !id.bytes().all(|b| b.is_ascii_digit()) {
        return Err(fail("invalid_post_id"));
    }
    Ok(id.to_string())
}

#[cfg(test)]
mod tests {
    use super::status_id;

    #[test]
    fn accepts_post_urls() {
        for (input, id) in [
            ("https://x.com/jack/status/20", "20"),
            ("  https://twitter.com/NASA/status/2051361508181856362/photo/4?s=20  ", "2051361508181856362"),
            ("https://mobile.twitter.com/a/status/123", "123"),
            ("https://x.com/i/web/status/123", "123"),
            ("http://WWW.X.COM/a/status/9", "9"),
        ] {
            assert_eq!(status_id(input).as_deref(), Ok(id), "{input}");
        }
    }

    #[test]
    fn rejects_everything_else() {
        for input in [
            "https://x.com.evil.com/a/status/1",
            "https://evil.com/x.com/a/status/1",
            "https://x.com/jack",
            "https://x.com/a/status/12ab",
            "https://x.com/a/status/../../etc",
            "file:///etc/passwd",
            "https://x.com/a/status/123456789012345678901",
            "no es una url",
        ] {
            assert!(status_id(input).is_err(), "{input}");
        }
    }

    #[test]
    fn errors_are_codes_not_sentences() {
        assert_eq!(status_id("no es una url"), Err("invalid_link".into()));
        assert_eq!(status_id("file:///etc/passwd"), Err("link_scheme".into()));
        assert_eq!(status_id("https://evil.com/a/status/1"), Err("host_not_allowed".into()));
        assert_eq!(status_id("https://x.com/jack"), Err("not_a_post".into()));
        assert_eq!(status_id("https://x.com/a/status/12ab"), Err("invalid_post_id".into()));
    }
}

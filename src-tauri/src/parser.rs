//! Parser puro: JSON de syndication → `Post`.
//! Sin red ni disco: recibe texto y devuelve datos, así se prueba con fixtures reales.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Author {
    pub handle: String,
    pub name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Photo,
    Video,
    Gif,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Media {
    pub kind: MediaKind,
    /// URL a la mejor calidad disponible (foto `name=orig`, vídeo MP4 de mayor bitrate)
    pub url: String,
    pub poster_url: Option<String>,
    pub alt_text: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Link {
    pub expanded_url: String,
    pub display_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Post {
    pub id: String,
    pub url: String,
    pub author: Author,
    /// ISO 8601; None si la fuente no da fecha exacta (oEmbed)
    pub created_at: Option<String>,
    pub text: String,
    /// true si X recorta el texto (posts largos): no fingimos tener el texto completo
    pub text_truncated: bool,
    pub media: Vec<Media>,
    pub links: Vec<Link>,
    pub in_reply_to: Option<String>,
    pub quoted: Option<Box<Post>>,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// `{}`: falta token, post borrado o privado
    Empty,
    /// X devolvió una "lápida" (post retirado)
    Tombstone,
    Malformed(String),
}

pub fn parse_tweet(json: &str) -> Result<Post, ParseError> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| ParseError::Malformed(e.to_string()))?;
    if value.get("__typename").and_then(|t| t.as_str()) == Some("TweetTombstone") {
        return Err(ParseError::Tombstone);
    }
    if value.get("id_str").is_none() {
        return Err(ParseError::Empty);
    }
    let raw: RawTweet = serde_json::from_value(value).map_err(|e| ParseError::Malformed(e.to_string()))?;
    Ok(raw.into_post())
}

/// Respaldo 1: salida de `yt-dlp --dump-json` (solo existe si el post tiene vídeo).
pub fn parse_ytdlp(json: &str) -> Result<Post, ParseError> {
    let raw: RawYtDlp = serde_json::from_str(json).map_err(|e| ParseError::Malformed(e.to_string()))?;
    let best = raw
        .formats
        .iter()
        .filter(|f| f.protocol.as_deref() == Some("https") && f.ext.as_deref() == Some("mp4"))
        .max_by(|a, b| a.tbr.unwrap_or(0.0).total_cmp(&b.tbr.unwrap_or(0.0)))
        .ok_or(ParseError::Empty)?;
    // yt-dlp no separa enlaces de medios: el t.co final es siempre el del vídeo
    let mut text = raw.description.trim_end().to_string();
    if let Some(pos) = text.rfind("https://t.co/") {
        if !text[pos..].contains(char::is_whitespace) {
            text.truncate(pos);
        }
    }
    Ok(Post {
        url: format!("https://x.com/{}/status/{}", raw.uploader_id, raw.display_id),
        id: raw.display_id,
        author: Author { handle: raw.uploader_id, name: raw.uploader, avatar_url: None },
        created_at: raw.timestamp.map(iso_from_unix),
        text: decode_entities(text.trim()),
        text_truncated: false,
        media: vec![Media {
            kind: MediaKind::Video,
            url: best.url.clone(),
            poster_url: raw.thumbnail,
            alt_text: None,
            width: best.width,
            height: best.height,
            duration_ms: raw.duration.map(|d| (d * 1000.0) as u64),
        }],
        links: vec![],
        in_reply_to: None,
        quoted: None,
    })
}

/// Texto completo de un post largo vía FxTwitter (tercero). Solo se usa para eso:
/// medios, autor y fecha siguen saliendo de X. Comprueba que el ID coincide.
pub fn parse_fxtwitter_text(json: &str, id: &str) -> Result<String, ParseError> {
    #[derive(Deserialize)]
    struct Fx {
        tweet: FxTweet,
    }
    #[derive(Deserialize)]
    struct FxTweet {
        id: String,
        text: String,
    }
    let fx: Fx = serde_json::from_str(json).map_err(|e| ParseError::Malformed(e.to_string()))?;
    if fx.tweet.id != id || fx.tweet.text.trim().is_empty() {
        return Err(ParseError::Empty);
    }
    Ok(fx.tweet.text.trim().to_string())
}

/// Respaldo 2: oEmbed. Solo trae texto y autor; los enlaces siguen como t.co.
/// El HTML es un fragmento pequeño y fijo de la API oficial (`<blockquote><p>texto</p>&mdash; Nombre (@user)…`),
/// no una página: se lee solo el primer `<p>`.
pub fn parse_oembed(json: &str, id: &str) -> Result<Post, ParseError> {
    let raw: RawOEmbed = serde_json::from_str(json).map_err(|e| ParseError::Malformed(e.to_string()))?;
    let start = raw.html.find("<p").and_then(|i| raw.html[i..].find('>').map(|j| i + j + 1)).ok_or(ParseError::Empty)?;
    let end = raw.html[start..].find("</p>").map(|j| start + j).ok_or(ParseError::Empty)?;
    let paragraph = raw.html[start..end].replace("<br>", "\n");
    // quitar etiquetas; los <a> de pic.twitter.com son la foto, no texto
    let mut text = String::new();
    let mut rest = paragraph.as_str();
    while let Some(open) = rest.find('<') {
        text.push_str(&rest[..open]);
        let close = rest[open..].find('>').map(|j| open + j + 1).unwrap_or(rest.len());
        let tag = &rest[open..close];
        rest = &rest[close..];
        if tag.starts_with("<a") {
            let inner_end = rest.find("</a>").unwrap_or(rest.len());
            let inner = &rest[..inner_end];
            if !inner.starts_with("pic.twitter.com") && !inner.starts_with("pic.x.com") {
                text.push_str(inner);
            }
            rest = rest.get(inner_end + 4..).unwrap_or("");
        }
    }
    text.push_str(rest);
    let handle = raw.author_url.rsplit('/').next().unwrap_or_default().to_string();
    Ok(Post {
        id: id.to_string(),
        url: format!("https://x.com/{handle}/status/{id}"),
        author: Author { handle, name: raw.author_name, avatar_url: None },
        created_at: None,
        text: decode_entities(text.trim()),
        text_truncated: false,
        media: vec![],
        links: vec![],
        in_reply_to: None,
        quoted: None,
    })
}

// --- Formas crudas del JSON (solo los campos que usamos) ---

#[derive(Deserialize)]
struct RawTweet {
    id_str: String,
    text: String,
    #[serde(default)]
    display_text_range: Option<[usize; 2]>,
    #[serde(default)]
    entities: RawEntities,
    created_at: Option<String>,
    user: RawUser,
    #[serde(default, rename = "mediaDetails")]
    media_details: Vec<RawMedia>,
    in_reply_to_status_id_str: Option<String>,
    quoted_tweet: Option<Box<RawTweet>>,
    note_tweet: Option<serde_json::Value>,
}

#[derive(Deserialize, Default)]
struct RawEntities {
    #[serde(default)]
    urls: Vec<RawUrl>,
    #[serde(default)]
    media: Vec<RawUrl>,
}

#[derive(Deserialize)]
struct RawUrl {
    url: String,
    #[serde(default)]
    expanded_url: String,
    #[serde(default)]
    display_url: String,
}

#[derive(Deserialize)]
struct RawUser {
    screen_name: String,
    name: String,
    profile_image_url_https: Option<String>,
}

#[derive(Deserialize)]
struct RawMedia {
    #[serde(rename = "type")]
    kind: String,
    media_url_https: String,
    ext_alt_text: Option<String>,
    original_info: Option<RawSize>,
    video_info: Option<RawVideoInfo>,
}

#[derive(Deserialize)]
struct RawSize {
    width: u32,
    height: u32,
}

#[derive(Deserialize)]
struct RawVideoInfo {
    duration_millis: Option<u64>,
    #[serde(default)]
    variants: Vec<RawVariant>,
}

#[derive(Deserialize)]
struct RawVariant {
    content_type: String,
    url: String,
    bitrate: Option<u64>,
}

#[derive(Deserialize)]
struct RawYtDlp {
    /// en yt-dlp `id` es el del vídeo; el del post es `display_id`
    display_id: String,
    #[serde(default)]
    description: String,
    uploader: String,
    uploader_id: String,
    timestamp: Option<i64>,
    duration: Option<f64>,
    thumbnail: Option<String>,
    #[serde(default)]
    formats: Vec<RawFormat>,
}

#[derive(Deserialize)]
struct RawFormat {
    url: String,
    ext: Option<String>,
    protocol: Option<String>,
    tbr: Option<f64>,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Deserialize)]
struct RawOEmbed {
    html: String,
    author_name: String,
    author_url: String,
}

impl RawTweet {
    fn into_post(self) -> Post {
        let text = clean_text(&self.text, self.display_text_range, &self.entities);
        let media = self.media_details.iter().filter_map(to_media).collect();
        Post {
            url: format!("https://x.com/{}/status/{}", self.user.screen_name, self.id_str),
            id: self.id_str,
            author: Author {
                handle: self.user.screen_name,
                name: self.user.name,
                avatar_url: self.user.profile_image_url_https,
            },
            created_at: self.created_at,
            text,
            text_truncated: self.note_tweet.is_some(),
            media,
            links: self
                .entities
                .urls
                .into_iter()
                .map(|u| Link { expanded_url: u.expanded_url, display_url: u.display_url })
                .collect(),
            in_reply_to: self.in_reply_to_status_id_str,
            quoted: self.quoted_tweet.map(|q| Box::new(q.into_post())),
        }
    }
}

/// Texto del post y nada más: rango visible, sin enlaces de medios, t.co expandidos.
fn clean_text(text: &str, range: Option<[usize; 2]>, entities: &RawEntities) -> String {
    // display_text_range cuenta code points (un emoji = 1), no bytes: por eso chars()
    let chars: Vec<char> = text.chars().collect();
    let [from, to] = range.unwrap_or([0, chars.len()]);
    let mut out: String = chars[from.min(chars.len())..to.min(chars.len())].iter().collect();
    for m in &entities.media {
        out = out.replace(&m.url, "");
    }
    for u in &entities.urls {
        if !u.expanded_url.is_empty() {
            out = out.replace(&u.url, &u.expanded_url);
        }
    }
    decode_entities(out.trim())
}

fn to_media(m: &RawMedia) -> Option<Media> {
    let kind = match m.kind.as_str() {
        "photo" => MediaKind::Photo,
        "video" => MediaKind::Video,
        "animated_gif" => MediaKind::Gif,
        _ => return None,
    };
    let (url, poster_url, duration_ms) = if kind == MediaKind::Photo {
        (format!("{}?name=orig", m.media_url_https), None, None)
    } else {
        let info = m.video_info.as_ref()?;
        let best = info
            .variants
            .iter()
            .filter(|v| v.content_type == "video/mp4")
            .max_by_key(|v| v.bitrate.unwrap_or(0))?;
        (best.url.clone(), Some(m.media_url_https.clone()), info.duration_millis)
    };
    Some(Media {
        kind,
        url,
        poster_url,
        alt_text: m.ext_alt_text.clone(),
        width: m.original_info.as_ref().map(|s| s.width),
        height: m.original_info.as_ref().map(|s| s.height),
        duration_ms,
    })
}

/// X escapa solo estas entidades en el texto (y oEmbed usa &#39;)
fn decode_entities(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// Segundos Unix → ISO 8601 UTC sin dependencias (algoritmo civil_from_days de H. Hinnant)
pub(crate) fn iso_from_unix(secs: i64) -> String {
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> Post {
        let path = format!("{}/../tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"));
        let json = std::fs::read_to_string(&path).expect("fixture");
        parse_tweet(&json).unwrap_or_else(|e| panic!("{name}: {e:?}"))
    }

    const ALL: [&str; 9] = [
        "text_only", "photo_1", "photos_4", "video", "gif", "quote", "pdf_link", "reply_mention", "emoji",
    ];

    #[test]
    fn no_fixture_leaks_tco_links_into_text() {
        // t.co es ruido de X: o se expande a la URL real o se quita (enlace de la foto)
        for name in ALL {
            let post = fixture(name);
            assert!(!post.text.contains("t.co/"), "{name}: {:?}", post.text);
            assert_eq!(post.text, post.text.trim(), "{name}: espacios sobrantes");
        }
    }

    #[test]
    fn text_only() {
        let p = fixture("text_only");
        assert_eq!(p.text, "just setting up my twttr");
        assert_eq!(p.author.handle, "jack");
        assert_eq!(p.url, "https://x.com/jack/status/20");
        assert!(p.media.is_empty());
        assert!(!p.text_truncated);
    }

    #[test]
    fn photo_link_is_removed_even_inside_display_range() {
        // En posts antiguos display_text_range incluye el t.co de la foto
        let p = fixture("photo_1");
        assert_eq!(p.text, "If only Bradley's arm was longer. Best photo ever. #oscars");
        assert_eq!(p.media.len(), 1);
        assert_eq!(p.media[0].kind, MediaKind::Photo);
        assert!(p.media[0].url.ends_with("BhxWutnCEAAtEQ6.jpg?name=orig"));
    }

    #[test]
    fn four_photos_in_order_and_links_expanded() {
        let p = fixture("photos_4");
        let ids: Vec<_> = p.media.iter().map(|m| m.url.split("/media/").nth(1).unwrap().to_string()).collect();
        assert_eq!(ids, [
            "HHfmYn1X0AA-tjt.jpg?name=orig",
            "HHfmZLUXcAAabwz.jpg?name=orig",
            "HHfmZ1sXkAACmSj.jpg?name=orig",
            "HHfmaoUW4AEPF-9.jpg?name=orig",
        ]);
        assert!(p.text.ends_with("highlights from the mission: https://go.nasa.gov/4usiN8W"));
        assert_eq!(p.links[0].expanded_url, "https://go.nasa.gov/4usiN8W");
    }

    #[test]
    fn video_picks_highest_bitrate_mp4_and_flags_truncated_text() {
        let p = fixture("video");
        assert_eq!(p.media.len(), 1);
        assert_eq!(p.media[0].kind, MediaKind::Video);
        assert!(p.media[0].url.ends_with("/vid/avc1/1828x1210/0B4vTK2y6e2J12C2.mp4"));
        assert_eq!(p.media[0].duration_ms, Some(19381));
        assert!(p.text_truncated, "post largo: syndication solo da los primeros 280");
    }

    #[test]
    fn gif_is_saved_as_mp4() {
        let p = fixture("gif");
        assert_eq!(p.media[0].kind, MediaKind::Gif);
        assert_eq!(p.media[0].url, "https://video.twimg.com/tweet_video/E3oaRJwWQAYxwM-.mp4");
        assert!(p.text.ends_with("Thanks!"));
    }

    #[test]
    fn quoted_post_is_nested_with_its_own_text() {
        let p = fixture("quote");
        let q = p.quoted.as_ref().expect("quoted");
        assert_eq!(q.author.handle, "fractaledmind");
        assert!(q.text.ends_with("as a blog post as well: https://fractaledmind.github.io/2024/10/16/sqlite-supercharges-rails/"));
        // el texto del citado no se mezcla con el del post
        assert!(!p.text.contains("speaker notes"));
        assert!(p.text.ends_with("https://simonwillison.net/2024/Oct/16/sqlite-rails-world-202/"));
        assert_eq!(p.media[0].alt_text.as_deref().map(|a| a.starts_with("Animated slide")), Some(true));
    }

    #[test]
    fn pdf_link_is_kept_as_link() {
        let p = fixture("pdf_link");
        assert!(p.text.ends_with("Download the full report (PDF): https://data.ntsb.gov/carol-repgen/api/Aviation/ReportMain/GenerateNewestReport/202401/pdf"));
        assert_eq!(p.links.len(), 1);
        assert!(p.media.is_empty());
    }

    #[test]
    fn reply_drops_leading_mentions() {
        let p = fixture("reply_mention");
        assert!(p.text.starts_with("Hey, lots of users noticed this"), "{:?}", p.text);
        assert_eq!(p.in_reply_to.as_deref(), Some("2027747365638091208"));
    }

    #[test]
    fn emoji_ranges_count_code_points_not_bytes() {
        let p = fixture("emoji");
        assert!(p.text.starts_with("Good morning, world! 🌎\n\nWe have"));
        assert!(p.text.ends_with("journey to the Moon."));
    }

    #[test]
    fn empty_response_is_an_error_not_an_empty_post() {
        assert_eq!(parse_tweet("{}"), Err(ParseError::Empty));
        assert_eq!(parse_tweet(r#"{"__typename":"TweetTombstone"}"#), Err(ParseError::Tombstone));
        assert!(matches!(parse_tweet("<html>"), Err(ParseError::Malformed(_))));
    }

    #[test]
    fn html_entities_are_decoded() {
        let json = r#"{"id_str":"1","text":"a &amp; b &lt;3","display_text_range":[0,15],"entities":{},"created_at":"x","user":{"screen_name":"u","name":"U"}}"#;
        assert_eq!(parse_tweet(json).unwrap().text, "a & b <3");
    }

    fn raw(name: &str) -> String {
        std::fs::read_to_string(format!("{}/../tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"))).unwrap()
    }

    #[test]
    fn ytdlp_fallback_keeps_video_and_drops_media_link() {
        let p = parse_ytdlp(&raw("ytdlp_video")).unwrap();
        assert_eq!(p.id, "1969515038512926823");
        assert_eq!(p.author.handle, "rauchg");
        assert_eq!(p.created_at.as_deref(), Some("2025-09-20T21:32:31Z"));
        assert!(p.text.ends_with("a foundational piece of"), "{:?}", p.text);
        assert_eq!(p.media.len(), 1);
        assert!(p.media[0].url.contains("1828x1210"), "{}", p.media[0].url);
    }

    #[test]
    fn oembed_fallback_reads_only_the_post_paragraph() {
        let p = parse_oembed(&raw("oembed_text"), "20").unwrap();
        assert_eq!(p.text, "just setting up my twttr");
        assert_eq!(p.author.handle, "jack");
        assert_eq!(p.author.name, "jack");
        assert_eq!(p.created_at, None);

        let p = parse_oembed(&raw("oembed_photos"), "2051361508181856362").unwrap();
        assert!(p.text.starts_with("Start your week with some new photos from Artemis II!\n\nThough"));
        assert!(p.text.contains("we're still"), "entidades decodificadas");
        assert!(!p.text.contains("pic.twitter.com"), "el enlace de la foto no es texto del post");
        assert!(!p.text.contains("&mdash;") && !p.text.contains("May 4, 2026"), "la firma no es texto del post");
    }

    #[test]
    fn fxtwitter_gives_full_text_of_long_post() {
        let short = fixture("video");
        let full = parse_fxtwitter_text(&raw("fxtwitter_long"), "1969515038512926823").unwrap();
        assert!(full.starts_with(&short.text[..200]), "mismo post: {full:?}");
        assert!(full.len() > short.text.len() + 200);
        assert!(full.ends_with("Demo: http://shell.turso.tech"), "enlaces ya expandidos: {full:?}");
        assert!(!full.contains("t.co/"));
        // otro post o respuesta HTML (post inexistente): no se acepta
        assert!(parse_fxtwitter_text(&raw("fxtwitter_long"), "20").is_err());
        assert!(parse_fxtwitter_text("<!DOCTYPE html>", "20").is_err());
    }
}

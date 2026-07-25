use crate::error::{HitsuError, HitsuResult};

/// Extract the base URL (scheme + authority) from a full URL string.
pub(crate) fn extract_base_url(url_str: &str) -> Option<String> {
    let parsed = url::Url::parse(url_str).ok()?;
    let base = format!("{}://{}", parsed.scheme(), parsed.authority());
    Some(base)
}

/// Try to find a favicon URL from an HTML page body.
pub(crate) fn find_favicon_in_html(base: &str, html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    for pattern in &[
        "rel=\"icon\"",
        "rel='icon'",
        "rel=\"shortcut icon\"",
        "rel='shortcut icon'",
    ] {
        let pos = lower.find(pattern)?;
        let link_start = html[..pos].rfind("<link").unwrap_or(0);
        let tag_end = html[pos..]
            .find('>')
            .map(|i| pos + i + 1)
            .unwrap_or(html.len());
        let tag = &html[link_start..tag_end];
        if let Some(href) = extract_html_attr(tag, "href") {
            return resolve_favicon_url(base, &href);
        }
    }
    None
}

/// Extract an attribute value from an HTML tag snippet.
fn extract_html_attr(tag: &str, attr: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let attr_eq = format!("{attr}=");
    let pos = lower.find(&attr_eq)?;
    let after = &tag[pos + attr_eq.len()..];
    let delim = after.chars().next()?;
    if delim == '"' || delim == '\'' {
        let end = after[1..].find(delim)?;
        Some(after[1..end + 1].to_string())
    } else {
        let end = after
            .find(|c: char| c.is_whitespace() || c == '>')
            .unwrap_or(after.len());
        if end > 0 {
            Some(after[..end].to_string())
        } else {
            None
        }
    }
}

/// Resolve a possibly-relative favicon URL against a base URL.
fn resolve_favicon_url(base: &str, href: &str) -> Option<String> {
    if href.starts_with("http://") || href.starts_with("https://") {
        Some(href.to_string())
    } else if href.starts_with("//") {
        let proto = if base.starts_with("https") {
            "https"
        } else {
            "http"
        };
        Some(format!("{proto}:{href}"))
    } else if href.starts_with('/') {
        Some(format!("{base}{href}"))
    } else {
        Some(format!("{base}/{href}"))
    }
}

/// Fetch a favicon from a website. Tries `/favicon.ico` first, then falls back
/// to parsing the page's `<link rel="icon">` tags.
/// Returns the raw image bytes on success.
pub(crate) async fn fetch_favicon(url_str: &str) -> HitsuResult<Vec<u8>> {
    let base_url =
        extract_base_url(url_str).ok_or_else(|| HitsuError::Custom("Invalid URL".into()))?;

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; Hitsu/1.0)")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| HitsuError::Custom(format!("Failed to create HTTP client: {e}")))?;

    // Try the standard /favicon.ico location first
    let favicon_url = format!("{base_url}/favicon.ico");
    let favicon_data = match client.get(&favicon_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            if content_type.starts_with("image/") {
                resp.bytes()
                    .await
                    .map(|b| b.to_vec())
                    .ok()
                    .filter(|data| !data.is_empty() && data.len() < 1024 * 1024)
            } else {
                None
            }
        }
        _ => None,
    };

    // Fallback: fetch the page and look for a favicon link
    let favicon_data = if let Some(data) = favicon_data {
        data
    } else {
        let page_html = match client.get(url_str).send().await {
            Ok(resp) if resp.status().is_success() => resp.text().await.ok(),
            _ => None,
        };

        let html = page_html.ok_or_else(|| {
            HitsuError::Custom("Could not fetch the web page to look for a favicon".into())
        })?;

        let fav_url = find_favicon_in_html(&base_url, &html)
            .ok_or_else(|| HitsuError::Custom("No favicon found on the page".into()))?;

        let resp = client
            .get(&fav_url)
            .send()
            .await
            .map_err(|e| HitsuError::Custom(format!("Failed to download favicon: {e}")))?;

        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| HitsuError::Custom(format!("Failed to read favicon: {e}")))?
    };

    if favicon_data.is_empty() || favicon_data.len() >= 1024 * 1024 {
        return Err(HitsuError::Custom(
            "Downloaded favicon is too large or empty".into(),
        ));
    }

    Ok(favicon_data)
}

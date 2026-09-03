use std::net::IpAddr;
use std::sync::Arc;

use crate::error::{HitsuError, HitsuResult};

const MAX_FAVICON_BYTES: usize = 1024 * 1024;
const MAX_HTML_BYTES: usize = 2 * 1024 * 1024;

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            !(ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_broadcast()
                || ip.is_multicast()
                || ip.is_unspecified())
        }
        IpAddr::V6(ip) => {
            if let Some(ipv4) = ip.to_ipv4_mapped() {
                return is_public_ip(IpAddr::V4(ipv4));
            }
            !(ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
                || ip.is_multicast())
        }
    }
}

fn is_safe_http_url(url: &url::Url) -> bool {
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    match url.host() {
        Some(url::Host::Domain(host)) => {
            let host = host.trim_end_matches('.');
            !host.eq_ignore_ascii_case("localhost") && !host.ends_with(".localhost")
        }
        Some(url::Host::Ipv4(ip)) => is_public_ip(IpAddr::V4(ip)),
        Some(url::Host::Ipv6(ip)) => is_public_ip(IpAddr::V6(ip)),
        None => false,
    }
}

/// Resolve hostnames once and give reqwest only public addresses. Filtering in
/// the resolver prevents a second DNS lookup from rebinding a validated name to
/// a loopback or private address before the connection is opened.
struct PublicDnsResolver;

impl reqwest::dns::Resolve for PublicDnsResolver {
    fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let host = name.as_str().to_string();
        Box::pin(async move {
            let addresses = tokio::net::lookup_host((host.as_str(), 0))
                .await?
                .filter(|address| is_public_ip(address.ip()))
                .collect::<Vec<_>>();
            if addresses.is_empty() {
                return Err(
                    std::io::Error::other("hostname did not resolve to a public address").into(),
                );
            }
            Ok(Box::new(addresses.into_iter()) as reqwest::dns::Addrs)
        })
    }
}

pub(crate) fn http_client() -> HitsuResult<reqwest::Client> {
    reqwest::Client::builder()
        // Proxies resolve hostnames remotely, bypassing PublicDnsResolver's
        // rebinding filter — never traverse user/system proxies.
        .no_proxy()
        .dns_resolver(Arc::new(PublicDnsResolver))
        .user_agent("Mozilla/5.0 (compatible; Hitsu/1.0)")
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 5 {
                attempt.error("too many redirects")
            } else if is_safe_http_url(attempt.url()) {
                attempt.follow()
            } else {
                attempt.stop()
            }
        }))
        .build()
        .map_err(|e| HitsuError::Custom(format!("Failed to create HTTP client: {e}")))
}

/// Extract the base URL (scheme + authority) from a full URL string.
pub(crate) fn extract_base_url(url_str: &str) -> Option<String> {
    let parsed = url::Url::parse(url_str).ok()?;
    if !is_safe_http_url(&parsed) {
        return None;
    }
    Some(format!("{}://{}", parsed.scheme(), parsed.authority()))
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
        let Some(pos) = lower.find(pattern) else {
            continue;
        };
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

/// Resolve a possibly-relative favicon URL against a base URL, then apply the
/// same local-address guard used for page URLs and redirects.
fn resolve_favicon_url(base: &str, href: &str) -> Option<String> {
    let base = url::Url::parse(base).ok()?;
    let resolved = base.join(href).ok()?;
    is_safe_http_url(&resolved).then(|| resolved.to_string())
}

async fn read_limited_response(
    mut response: reqwest::Response,
    max_bytes: usize,
    description: &str,
) -> HitsuResult<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(HitsuError::Custom(format!("{description} is too large")));
    }

    let mut data = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| HitsuError::Custom(format!("Failed to read {description}: {error}")))?
    {
        if data.len().saturating_add(chunk.len()) > max_bytes {
            return Err(HitsuError::Custom(format!("{description} is too large")));
        }
        data.extend_from_slice(&chunk);
    }
    Ok(data)
}

/// Fetch a favicon from a website. Tries `/favicon.ico` first, then falls back
/// to parsing the page's `<link rel="icon">` tags.
/// Returns the raw image bytes on success.
pub(crate) async fn fetch_favicon(client: &reqwest::Client, url_str: &str) -> HitsuResult<Vec<u8>> {
    let base_url = extract_base_url(url_str)
        .ok_or_else(|| HitsuError::Custom("URL is invalid or points to a local address".into()))?;

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
                read_limited_response(resp, MAX_FAVICON_BYTES, "Downloaded favicon")
                    .await
                    .ok()
                    .filter(|data| !data.is_empty())
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
            Ok(resp) if resp.status().is_success() => {
                read_limited_response(resp, MAX_HTML_BYTES, "Web page")
                    .await
                    .ok()
            }
            _ => None,
        };

        let html = page_html.ok_or_else(|| {
            HitsuError::Custom("Could not fetch the web page to look for a favicon".into())
        })?;
        let html = String::from_utf8_lossy(&html);

        let fav_url = find_favicon_in_html(&base_url, &html)
            .ok_or_else(|| HitsuError::Custom("No favicon found on the page".into()))?;

        let resp = client
            .get(&fav_url)
            .send()
            .await
            .map_err(|e| HitsuError::Custom(format!("Failed to download favicon: {e}")))?;
        if !resp.status().is_success() {
            return Err(HitsuError::Custom(format!(
                "Favicon download returned HTTP {}",
                resp.status()
            )));
        }
        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");
        if !content_type.starts_with("image/") {
            return Err(HitsuError::Custom(
                "Favicon response is not an image".into(),
            ));
        }

        read_limited_response(resp, MAX_FAVICON_BYTES, "Downloaded favicon").await?
    };

    if favicon_data.is_empty() || favicon_data.len() > MAX_FAVICON_BYTES {
        return Err(HitsuError::Custom(
            "Downloaded favicon is too large or empty".into(),
        ));
    }

    Ok(favicon_data)
}

#[cfg(test)]
mod tests {
    use super::{extract_base_url, fetch_favicon, find_favicon_in_html, MAX_FAVICON_BYTES};
    use std::io::{Read, Write};
    use std::time::{Duration, Instant};

    #[test]
    fn rejects_local_and_non_http_urls() {
        assert!(extract_base_url("http://127.0.0.1/login").is_none());
        assert!(extract_base_url("http://[::1]/login").is_none());
        assert!(extract_base_url("http://[::ffff:127.0.0.1]/login").is_none());
        assert!(extract_base_url("http://localhost/login").is_none());
        assert!(extract_base_url("file:///tmp/icon.png").is_none());
        assert_eq!(
            extract_base_url("https://example.com/login").as_deref(),
            Some("https://example.com")
        );
    }

    #[test]
    fn rejects_local_absolute_favicon_url_from_remote_page() {
        let html = r#"<link rel="icon" href="http://127.0.0.1:8080/private">"#;

        assert_eq!(find_favicon_in_html("https://example.com", html), None);
    }

    #[test]
    fn tries_each_supported_icon_relation() {
        let html = "<html><head><link rel='shortcut icon' href='/brand.ico'></head></html>";
        assert_eq!(
            find_favicon_in_html("https://example.com", html).as_deref(),
            Some("https://example.com/brand.ico")
        );
    }

    #[tokio::test]
    async fn rejects_oversized_favicon_from_headers_without_reading_body() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        std::thread::spawn(move || {
            let started = Instant::now();
            while started.elapsed() < Duration::from_secs(3) {
                let Ok((mut stream, _)) = listener.accept() else {
                    std::thread::sleep(Duration::from_millis(5));
                    continue;
                };
                std::thread::spawn(move || {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(1)))
                        .unwrap();
                    let mut request = [0u8; 4096];
                    let bytes_read = stream.read(&mut request).unwrap_or(0);
                    let request = String::from_utf8_lossy(&request[..bytes_read]);
                    if request.starts_with("GET /favicon.ico ") {
                        write!(
                            stream,
                            "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\n\
                             Content-Length: {}\r\nConnection: close\r\n\r\n",
                            MAX_FAVICON_BYTES + 1
                        )
                        .unwrap();
                        stream.flush().unwrap();
                        std::thread::sleep(Duration::from_secs(2));
                    } else {
                        let body = "<html></html>";
                        write!(
                            stream,
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\
                             Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        )
                        .unwrap();
                    }
                });
            }
        });

        let client = reqwest::Client::builder()
            .no_proxy()
            .resolve("favicon-size.test", address)
            .build()
            .unwrap();
        let url = format!("http://favicon-size.test:{}/", address.port());
        let fetch = fetch_favicon(&client, &url);

        assert!(
            tokio::time::timeout(Duration::from_millis(500), fetch)
                .await
                .is_ok(),
            "oversized favicon should be rejected from Content-Length without reading its body"
        );
    }
}

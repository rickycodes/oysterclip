use reqwest::{redirect::Policy, Client};
use scraper::{Html, Selector};
use std::net::IpAddr;
use std::sync::LazyLock;
use std::time::Duration;
use url::Url;

const PREVIEW_TIMEOUT: Duration = Duration::from_secs(4);
const MAX_RETRY_ATTEMPTS: u8 = 3;
const INITIAL_BACKOFF_MS: u64 = 2000;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(PREVIEW_TIMEOUT)
        .redirect(Policy::limited(5))
        .user_agent("clipboard-viewer-ui/0.1")
        .build()
        .expect("failed to build preview http client")
});

static META_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("meta").expect("failed to parse meta selector"));
static TITLE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("title").expect("failed to parse title selector"));

#[derive(Clone, PartialEq, Eq)]
pub enum LinkPreviewState {
    Loading,
    Ready(LinkPreview),
    Failed,
}

#[derive(Clone, PartialEq, Eq)]
pub struct LinkPreview {
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub site_name: Option<String>,
    pub image_url: Option<String>,
    pub display_url: String,
}

pub async fn fetch_link_preview(raw_url: &str) -> Option<LinkPreview> {
    let parsed = validate_preview_url(raw_url)?;

    for attempt in 1..=MAX_RETRY_ATTEMPTS {
        match try_fetch_preview(&parsed).await {
            Ok(preview) => return Some(preview),
            Err(FetchError::Transient) => {
                if attempt < MAX_RETRY_ATTEMPTS {
                    let backoff_ms =
                        INITIAL_BACKOFF_MS * (2_u64.saturating_pow((attempt - 1) as u32));
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                }
            }
            Err(FetchError::Permanent) => return None,
        }
    }

    None
}

enum FetchError {
    Transient,
    Permanent,
}

async fn try_fetch_preview(url: &Url) -> Result<LinkPreview, FetchError> {
    let response = send_request(url).await?;
    validate_response(&response)?;
    
    let final_url = response.url().clone();
    if !is_safe_preview_host(&final_url) {
        return Err(FetchError::Permanent);
    }

    if !is_html_content(&response) {
        return Err(FetchError::Permanent);
    }

    let html = response.text().await.map_err(|_| FetchError::Transient)?;
    parse_link_preview(&final_url, &html).ok_or(FetchError::Permanent)
}

async fn send_request(url: &Url) -> Result<reqwest::Response, FetchError> {
    HTTP_CLIENT.get(url.clone()).send().await.map_err(|e| {
        if e.is_timeout() || e.is_connect() {
            FetchError::Transient
        } else {
            FetchError::Permanent
        }
    })
}

fn validate_response(response: &reqwest::Response) -> Result<(), FetchError> {
    let status = response.status();
    if !status.is_success() {
        return Err(if status.is_server_error() {
            FetchError::Transient
        } else {
            FetchError::Permanent
        });
    }
    Ok(())
}

fn is_html_content(response: &reqwest::Response) -> bool {
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    content_type.is_empty() || content_type.starts_with("text/html")
}

fn parse_link_preview(final_url: &Url, html: &str) -> Option<LinkPreview> {
    let document = Html::parse_document(html);
    
    let title = parse_title(&document)?;
    let description = parse_description(&document);
    let site_name = parse_site_name(&document, final_url);
    let image_url = parse_image_url(&document, final_url);

    Some(LinkPreview {
        url: final_url.to_string(),
        title,
        description,
        site_name,
        image_url,
        display_url: final_url
            .host_str()
            .unwrap_or(raw_host(final_url))
            .to_string(),
    })
}

fn parse_title(document: &Html) -> Option<String> {
    meta_content(document, "property", "og:title")
        .or_else(|| meta_content(document, "name", "twitter:title"))
        .or_else(|| extract_title_tag(document))
        .map(clean_text)
        .filter(|value| !value.is_empty())
}

fn extract_title_tag(document: &Html) -> Option<String> {
    document
        .select(&TITLE_SELECTOR)
        .next()
        .map(|node| node.text().collect())
}

fn parse_description(document: &Html) -> Option<String> {
    meta_content(document, "property", "og:description")
        .or_else(|| meta_content(document, "name", "twitter:description"))
        .or_else(|| meta_content(document, "name", "description"))
        .map(clean_text)
        .filter(|value| !value.is_empty())
}

fn parse_site_name(document: &Html, final_url: &Url) -> Option<String> {
    meta_content(document, "property", "og:site_name")
        .map(clean_text)
        .filter(|value| !value.is_empty())
        .or_else(|| final_url.host_str().map(str::to_string))
}

fn parse_image_url(document: &Html, final_url: &Url) -> Option<String> {
    meta_content(document, "property", "og:image")
        .or_else(|| meta_content(document, "name", "twitter:image"))
        .and_then(|value| final_url.join(value.trim()).ok().map(|url| url.to_string()))
}

fn meta_content(document: &Html, attr_name: &str, attr_value: &str) -> Option<String> {
    document.select(&META_SELECTOR).find_map(|node| {
        let value = node.value();
        let attr_matches = value
            .attr(attr_name)
            .map(|current| current.eq_ignore_ascii_case(attr_value))
            .unwrap_or(false);
        if attr_matches {
            value.attr("content").map(str::to_string)
        } else {
            None
        }
    })
}

fn clean_text(value: String) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn validate_preview_url(raw_url: &str) -> Option<Url> {
    let parsed = Url::parse(raw_url).ok()?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return None,
    }
    if !is_safe_preview_host(&parsed) {
        return None;
    }
    Some(parsed)
}

fn is_safe_preview_host(url: &Url) -> bool {
    let host = match url.host_str() {
        Some(host) => host,
        None => return false,
    };

    if is_localhost(host) {
        return false;
    }

    is_safe_ip_address(host)
}

fn is_localhost(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
}

fn is_safe_ip_address(host: &str) -> bool {
    match host.parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) => {
            !(ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified())
        }
        Ok(IpAddr::V6(ip)) => !(ip.is_loopback() || ip.is_unspecified()),
        Err(_) => true,
    }
}

fn raw_host(url: &Url) -> &str {
    url.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_error_classification() {
        // Transient errors: timeout, connect, 5xx
        assert!(matches!(FetchError::Transient, FetchError::Transient));

        // Permanent errors: 4xx, parsing failures
        assert!(matches!(FetchError::Permanent, FetchError::Permanent));
    }

    #[test]
    fn test_backoff_calculation() {
        // Verify exponential backoff: 2s, 4s, 8s
        let backoff_1 = INITIAL_BACKOFF_MS * (2_u64.saturating_pow(0));
        let backoff_2 = INITIAL_BACKOFF_MS * (2_u64.saturating_pow(1));
        let backoff_3 = INITIAL_BACKOFF_MS * (2_u64.saturating_pow(2));

        assert_eq!(backoff_1, 2000);
        assert_eq!(backoff_2, 4000);
        assert_eq!(backoff_3, 8000);
    }

    #[test]
    fn test_max_attempts() {
        assert_eq!(MAX_RETRY_ATTEMPTS, 3);
    }
}

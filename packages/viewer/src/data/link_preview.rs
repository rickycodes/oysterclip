use reqwest::{redirect::Policy, Client};
use scraper::{Html, Selector};
use std::net::IpAddr;
use std::sync::LazyLock;
use std::time::Duration;
use url::Url;

const PREVIEW_TIMEOUT: Duration = Duration::from_secs(4);

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
    let response = HTTP_CLIENT.get(parsed.clone()).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }

    let final_url = response.url().clone();
    if !is_safe_preview_host(&final_url) {
        return None;
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !content_type.is_empty() && !content_type.starts_with("text/html") {
        return None;
    }

    let html = response.text().await.ok()?;
    parse_link_preview(&final_url, &html)
}

fn parse_link_preview(final_url: &Url, html: &str) -> Option<LinkPreview> {
    let document = Html::parse_document(html);
    let title = meta_content(&document, "property", "og:title")
        .or_else(|| meta_content(&document, "name", "twitter:title"))
        .or_else(|| {
            document
                .select(&TITLE_SELECTOR)
                .next()
                .map(|node| node.text().collect())
        })
        .map(clean_text)
        .filter(|value| !value.is_empty())?;

    let description = meta_content(&document, "property", "og:description")
        .or_else(|| meta_content(&document, "name", "twitter:description"))
        .or_else(|| meta_content(&document, "name", "description"))
        .map(clean_text)
        .filter(|value| !value.is_empty());

    let site_name = meta_content(&document, "property", "og:site_name")
        .map(clean_text)
        .filter(|value| !value.is_empty())
        .or_else(|| final_url.host_str().map(str::to_string));

    let image_url = meta_content(&document, "property", "og:image")
        .or_else(|| meta_content(&document, "name", "twitter:image"))
        .and_then(|value| final_url.join(value.trim()).ok().map(|url| url.to_string()));

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

    if host.eq_ignore_ascii_case("localhost") {
        return false;
    }

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

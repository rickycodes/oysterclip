pub fn is_image_data_uri(content: &str) -> bool {
    let trimmed = content.trim();
    trimmed.starts_with("data:image/") && trimmed.contains(";base64,")
}

pub fn image_data_uri_summary(content: &str) -> String {
    let trimmed = content.trim();
    let media_type = trimmed
        .strip_prefix("data:")
        .and_then(|value| value.split(';').next())
        .unwrap_or("image data");
    format!(
        "{} hidden for readability ({} chars)",
        media_type,
        trimmed.chars().count()
    )
}

pub fn extract_html_img_src(html: &str) -> Option<String> {
    let html = html.trim();
    if !html.starts_with("<img") || !html.ends_with("/>") {
        return None;
    }

    let src_start = html.find("src=")?;
    let after_src = &html[src_start + 4..];

    let url = if let Some(after_quote) = after_src.strip_prefix('"') {
        after_quote.split('"').next()?
    } else if let Some(after_quote) = after_src.strip_prefix('\'') {
        after_quote.split('\'').next()?
    } else {
        // Unquoted URL
        after_src.split([' ', '>', '/']).next()?
    };

    if url.is_empty() {
        return None;
    }

    Some(url.to_string())
}

pub fn is_html_img_tag(text: &str) -> bool {
    text.starts_with("<img") && text.ends_with("/>") && text.contains("src=")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_html_img_src_double_quotes() {
        let html = r#"<img src="https://example.com/image.png"/>"#;
        assert_eq!(
            extract_html_img_src(html),
            Some("https://example.com/image.png".to_string())
        );
    }

    #[test]
    fn test_extract_html_img_src_single_quotes() {
        let html = r#"<img src='https://example.com/image.jpg'/>"#;
        assert_eq!(
            extract_html_img_src(html),
            Some("https://example.com/image.jpg".to_string())
        );
    }

    #[test]
    fn test_is_html_img_tag() {
        assert!(is_html_img_tag(
            r#"<img src="https://example.com/image.png"/>"#
        ));
        assert!(!is_html_img_tag(
            r#"<img src="https://example.com/image.png">"#
        ));
        assert!(!is_html_img_tag("https://example.com/image.png"));
    }
}

use zxcvbn::Score;

const PASSWORD_LEN: usize = 25;
const PASSWORD_PREVIEW_MASK_LEN: usize = 8;

// Simple URL detection for filtering out URLs from password classification
fn contains_url(text: &str) -> bool {
    text.contains("http://") || text.contains("https://") || text.contains("www.")
}

pub fn is_password(text: &str) -> bool {
    text.len() == PASSWORD_LEN
        && !text.contains(' ')
        && !text.contains('\n')
        && !text.contains('\t')
        && !contains_url(text)
        && zxcvbn::zxcvbn(text, &[]).score() >= Score::Three
}

pub fn mask_password() -> String {
    "•".repeat(PASSWORD_LEN)
}

pub fn mask_password_preview() -> String {
    "•".repeat(PASSWORD_PREVIEW_MASK_LEN)
}

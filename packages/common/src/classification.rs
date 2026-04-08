const PASSWORD_LEN: usize = 25;
const PASSWORD_PREVIEW_MASK_LEN: usize = 8;

pub fn is_password(text: &str) -> bool {
    text.len() == PASSWORD_LEN
        && !text.contains(' ')
        && !text.contains('\n')
        && !text.contains('\t')
}

pub fn mask_password() -> String {
    "•".repeat(PASSWORD_LEN)
}

pub fn mask_password_preview() -> String {
    "•".repeat(PASSWORD_PREVIEW_MASK_LEN)
}

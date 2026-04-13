// Re-export all formatting utilities
pub use classification::{entry_icon_name, entry_label, preview_text};
pub use common::classification::{is_password, mask_password};
pub use image::{extract_html_img_src, image_data_uri_summary, is_html_img_tag, is_image_data_uri};
pub use timestamp::{format_relative_timestamp, format_timestamp};
pub use url::{extract_single_url, has_urls, split_text_with_urls, TextSegment};

mod classification;
mod image;
mod text_type;
mod timestamp;
mod url;

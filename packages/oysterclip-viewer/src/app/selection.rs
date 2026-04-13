use crate::app::actions::entry_id;
use crate::app::query::matches_query;
use crate::data::entry::ClipboardEntry;
use crate::data::format::entry_label;
use crate::ui::DetailState;
use crate::config::settings::PasswordConfig;

/// Represents the current selection state derived from filtered entries and selected ID.
/// Encapsulates all computed data needed to render the detail view.
#[derive(Clone)]
pub struct SelectionSnapshot {
    pub filtered_entries: Vec<ClipboardEntry>,
    pub current_selected_id: Option<i64>,
    pub selected_text: Option<String>,
    pub selected_label: &'static str,
    pub detail_state: DetailState,
}

impl SelectionSnapshot {
    /// Compute the selection snapshot from current state.
    /// Encapsulates all the conditional logic for filtering and extracting selected entry data.
    pub fn compute(
        current_entries: &[ClipboardEntry],
        current_query: &str,
        current_selected_id: Option<i64>,
        error: Option<&str>,
        password_config: PasswordConfig,
    ) -> Self {
        let filtered_entries: Vec<ClipboardEntry> = current_entries
            .iter()
            .filter(|entry| matches_query(entry, current_query, &password_config))
            .cloned()
            .collect();

        let selected_entry = current_selected_id.and_then(|id| {
            filtered_entries
                .iter()
                .find(|entry| entry_id(entry) == id)
                .cloned()
        });

        let selected_text = current_selected_id.and_then(|id| {
            filtered_entries.iter().find_map(|entry| match entry {
                ClipboardEntry::Text {
                    id: entry_id,
                    content,
                    ..
                } if *entry_id == id => Some(content.clone()),
                _ => None,
            })
        });

        let selected_label = current_selected_id
            .and_then(|id| filtered_entries.iter().find(|e| entry_id(e) == id))
            .map(|e| entry_label(e, &password_config))
            .unwrap_or("Text");

        let detail_state = if let Some(message) = error {
            DetailState::Error(message.to_string())
        } else if current_entries.is_empty() {
            DetailState::EmptyHistory
        } else if filtered_entries.is_empty() {
            DetailState::EmptySearch(current_query.to_string())
        } else if let Some(entry) = selected_entry {
            DetailState::Entry(entry)
        } else {
            DetailState::Unselected
        };

        Self {
            filtered_entries,
            current_selected_id,
            selected_text,
            selected_label,
            detail_state,
        }
    }
}

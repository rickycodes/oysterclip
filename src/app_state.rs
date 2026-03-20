use dioxus::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::app_actions::{entry_id, matches_query};
use crate::components::DetailState;
use crate::entry::{CachedEntries, ClipboardEntry, ClipboardPayload};
use crate::history::get_clipboard_entries;
use crate::source::ClipboardSource;

pub struct AppState {
    pub source: Arc<ClipboardSource>,
    pub cache: Arc<Mutex<Option<CachedEntries>>>,
    pub entries: Signal<Vec<ClipboardEntry>>,
    pub selected_id: Signal<Option<i64>>,
    pub query: Signal<String>,
    pub error: Signal<Option<String>>,
    pub copy_status: Signal<Option<String>>,
    pub action_status: Signal<Option<String>>,
    pub filtered_entries: Vec<ClipboardEntry>,
    pub current_selected_id: Option<i64>,
    pub current_query: String,
    pub total_entries: usize,
    pub detail_state: DetailState,
    pub selected_text: Option<String>,
}

pub fn use_app_state() -> AppState {
    let source = use_hook(|| Arc::new(ClipboardSource::from_env()));
    let cache = use_hook(|| Arc::new(Mutex::new(None::<CachedEntries>)));
    let mut entries = use_signal(Vec::<ClipboardEntry>::new);
    let mut selected_id = use_signal(|| None::<i64>);
    let query = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let copy_status = use_signal(|| None::<String>);
    let action_status = use_signal(|| None::<String>);

    let polling_source = source.clone();
    let polling_cache = cache.clone();

    use_future(move || {
        let source = polling_source.clone();
        let cache = polling_cache.clone();
        async move {
            loop {
                let payload = {
                    if let Ok(mut cache_guard) = cache.lock() {
                        get_clipboard_entries(&source, &mut cache_guard)
                    } else {
                        ClipboardPayload {
                            entries: Vec::new(),
                            error: Some("Failed to acquire cache lock.".to_string()),
                        }
                    }
                };

                if let Some(err) = payload.error {
                    if error() != Some(err.clone()) {
                        error.set(Some(err));
                    }
                } else {
                    if error().is_some() {
                        error.set(None);
                    }

                    let new_entries = payload.entries;
                    let new_len = new_entries.len();
                    let selected_still_exists = selected_id()
                        .map(|id| new_entries.iter().any(|entry| entry_id(entry) == id))
                        .unwrap_or(false);
                    if entries() != new_entries {
                        entries.set(new_entries);
                    }

                    if !selected_still_exists || new_len == 0 {
                        selected_id.set(None);
                    }
                }

                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    });

    let current_entries = entries();
    let current_query = query();
    let filtered_entries: Vec<ClipboardEntry> = current_entries
        .iter()
        .filter(|entry| matches_query(entry, &current_query))
        .cloned()
        .collect();
    let current_selected_id = selected_id();
    let detail = current_selected_id.and_then(|id| {
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
    let detail_state = if let Some(message) = error() {
        DetailState::Error(message)
    } else if current_entries.is_empty() {
        DetailState::EmptyHistory
    } else if filtered_entries.is_empty() {
        DetailState::EmptySearch(current_query.clone())
    } else if let Some(entry) = detail {
        DetailState::Entry(entry)
    } else {
        DetailState::Unselected
    };

    AppState {
        source,
        cache,
        entries,
        selected_id,
        query,
        error,
        copy_status,
        action_status,
        filtered_entries,
        current_selected_id,
        current_query,
        total_entries: current_entries.len(),
        detail_state,
        selected_text,
    }
}

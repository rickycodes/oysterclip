use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::app_actions::{entry_id, matches_query};
use crate::auth::AuthCache;
use crate::components::DetailState;
use crate::entry::{CachedEntries, ClipboardEntry, ClipboardPayload};
use crate::format::extract_single_url;
use crate::history::get_clipboard_entries;
use crate::link_preview::{fetch_link_preview, LinkPreviewState};
use crate::source::ClipboardSource;
use crate::watcher_control::{self, WatcherStatus};

const PREFETCH_URL_LIMIT: usize = 16;
const PREFETCH_IDLE_MS: u64 = 800;
const PREFETCH_STEP_MS: u64 = 75;

pub struct AppState {
    pub source: Arc<ClipboardSource>,
    pub cache: Arc<Mutex<Option<CachedEntries>>>,
    pub entries: Signal<Vec<ClipboardEntry>>,
    pub selected_id: Signal<Option<i64>>,
    pub selected_ids: Signal<HashSet<i64>>,
    pub query: Signal<String>,
    pub error: Signal<Option<String>>,
    pub copy_status: Signal<Option<(i64, String)>>,
    pub action_status: Signal<Option<String>>,
    pub show_password: Signal<bool>,
    pub auth_cache: Signal<Arc<Mutex<AuthCache>>>,
    pub watcher_status: Signal<WatcherStatus>,
    pub link_previews: Signal<HashMap<String, LinkPreviewState>>,
    pub filtered_entries: Vec<ClipboardEntry>,
    pub current_selected_id: Option<i64>,
    pub current_query: String,
    pub total_entries: usize,
    pub detail_state: DetailState,
    pub selected_text: Option<String>,
    pub current_watcher_status: WatcherStatus,
}

pub fn use_app_state() -> AppState {
    let source = use_hook(|| Arc::new(ClipboardSource::from_env()));
    let cache = use_hook(|| Arc::new(Mutex::new(None::<CachedEntries>)));
    let mut entries = use_signal(Vec::<ClipboardEntry>::new);
    let mut selected_id = use_signal(|| None::<i64>);
    let selected_ids = use_signal(HashSet::<i64>::new);
    let query = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let copy_status = use_signal(|| None::<(i64, String)>);
    let action_status = use_signal(|| None::<String>);
    let show_password = use_signal(|| false);
    let auth_cache = use_signal(|| Arc::new(Mutex::new(AuthCache::new(5))));
    let mut watcher_status =
        use_signal(|| WatcherStatus::unavailable("Waiting for watcher status."));
    let mut link_previews = use_signal(HashMap::<String, LinkPreviewState>::new);

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

    let watcher_source = source.clone();
    use_future(move || {
        let source = watcher_source.clone();
        async move {
            loop {
                watcher_status.set(watcher_control::get_status(&source));
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    });

    let preview_entries = entries;
    let preview_query = query;
    use_future(move || async move {
        loop {
            let eligible_urls: Vec<String> = preview_entries()
                .iter()
                .filter(|entry| matches_query(entry, &preview_query()))
                .filter_map(|entry| match entry {
                    ClipboardEntry::Text { content, .. } => {
                        extract_single_url(content).map(str::to_string)
                    }
                    ClipboardEntry::Image { .. } => None,
                })
                .take(PREFETCH_URL_LIMIT)
                .collect();

            let next_url = {
                let cache = link_previews();
                eligible_urls
                    .into_iter()
                    .find(|url| !cache.contains_key(url))
            };

            if let Some(url) = next_url {
                let mut cache = link_previews();
                cache.insert(url.clone(), LinkPreviewState::Loading);
                link_previews.set(cache);

                let next_state = fetch_link_preview(&url)
                    .await
                    .map(LinkPreviewState::Ready)
                    .unwrap_or(LinkPreviewState::Failed);
                let mut cache = link_previews();
                cache.insert(url, next_state);
                link_previews.set(cache);

                tokio::time::sleep(Duration::from_millis(PREFETCH_STEP_MS)).await;
            } else {
                tokio::time::sleep(Duration::from_millis(PREFETCH_IDLE_MS)).await;
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
        selected_ids,
        query,
        error,
        copy_status,
        action_status,
        show_password,
        auth_cache,
        watcher_status,
        link_previews,
        filtered_entries,
        current_selected_id,
        current_query,
        total_entries: current_entries.len(),
        detail_state,
        selected_text,
        current_watcher_status: watcher_status(),
    }
}

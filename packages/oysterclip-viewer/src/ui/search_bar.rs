use dioxus::prelude::*;
use std::rc::Rc;

#[component]
pub fn SearchBar(
    query: String,
    focus_search: ReadSignal<u32>,
    on_query_input: EventHandler<String>,
) -> Element {
    let mut search_input: Signal<Option<Rc<MountedData>>> = use_signal(|| None);

    use_effect(move || {
        let count = focus_search();
        if count > 0 {
            let el = search_input();
            if let Some(el) = el {
                spawn(async move {
                    let _ = el.set_focus(true).await;
                });
            }
        }
    });

    rsx! {
        div { class: "sidebar-search",
            input {
                class: "sidebar-search-input",
                r#type: "search",
                placeholder: "Search history",
                value: "{query}",
                onmounted: move |e| search_input.set(Some(e.data())),
                oninput: move |event| on_query_input.call(event.value().to_string()),
                onkeydown: move |event| {
                    if event.code() == Code::Escape {
                        on_query_input.call(String::new());
                        let el = search_input();
                        if let Some(el) = el {
                            spawn(async move {
                                let _ = el.set_focus(false).await;
                            });
                        }
                    }
                    // Stop propagation to prevent app-level shortcuts
                    // while the search input is focused
                    event.stop_propagation();
                },
            }
        }
    }
}

use dioxus::prelude::*;

#[component]
pub fn ImageOverlay(
    src: String,
    is_open: bool,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: if is_open { "image-overlay is-open" } else { "image-overlay" },
            onclick: move |_| on_close.call(()),
            aria_hidden: if is_open { "false" } else { "true" },
            button {
                class: "image-overlay-close",
                onclick: move |_| on_close.call(()),
                aria_label: "Close image overlay",
                tabindex: if is_open { "0" } else { "-1" },
                svg {
                    class: "image-overlay-close-icon",
                    view_box: "0 0 24 24",
                    width: "44",
                    height: "44",
                    stroke_width: "2",
                    stroke: "currentColor",
                    fill: "none",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path {
                        d: "M18 6L6 18M6 6l12 12",
                    }
                }
            }
            div {
                class: "image-overlay-dialog",
                img {
                    class: "image-overlay-image",
                    src,
                    alt: "Clipboard image expanded",
                    onclick: move |event: MouseEvent| event.stop_propagation(),
                }
            }
        }
    }
}

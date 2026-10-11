use dioxus::prelude::*;

use super::source::Source;

#[derive(PartialEq, Props, Clone)]
pub struct Props {
    pub message: Element,
}

#[component]
pub fn Warning(props: Props) -> Element {
    rsx! {
        div { class: "admonition admonition-warning", {props.message} }
    }
}

#[component]
pub fn Error(props: Props) -> Element {
    rsx! {
        div { class: "admonition admonition-error", {props.message} }
    }
}

#[component]
pub fn Success(props: Props) -> Element {
    rsx! {
        div { class: "admonition admonition-success", {props.message} }
    }
}

#[component]
pub fn Note(props: Props) -> Element {
    rsx! {
        div { class: "admonition admonition-note", {props.message} }
    }
}

#[component]
pub fn Main() -> Element {
    let mut messages = use_signal(circ_buffer::RingBuffer::<_, 8>::new);

    let update_form = move |event: Event<FormData>| async move {
        event.prevent_default();

        let values: std::collections::HashMap<_, _> = event
            .data
            .values()
            .iter()
            .filter_map(|(k, v)| {
                if let FormValue::Text(t) = v {
                    Some((k.clone(), t.clone()))
                } else {
                    None
                }
            })
            .collect();

        let source = values
            .get("search_type")
            .and_then(|s| Source::from_id(s))
            .unwrap_or(Source::Crate);
        let input = values
            .get("input_text")
            .cloned()
            .unwrap_or_default();

        for message in source.run(&input).await {
            messages.write().push(message);
        }
    };

    rsx! {
        div { id: "hero", class: "middle",
            h1 { "x2bib" }
            h3 { "Create a BibLaTeX entry from a crate, PyPI/Julia package, DOI, archive.org item or ISBN" }
            form { onsubmit: update_form,
                input {
                    name: "input_text",
                    r#type: "text",
                    value: "cellular-raza@0.4",
                }
                select { name: "search_type",
                    for source in Source::ALL {
                        option { value: source.id(), {source.id()} }
                    }
                }
                input { value: "search", r#type: "submit" }
            }
            h2 { "BibLaTeX Citation" }

            for i in 0..messages.read().len() {
                div { style: "margin: 0.5em;",
                    {&messages.read()[messages.read().len() - i - 1].clone()}
                }
            }

            for source in Source::ALL {
                {source.description()}
            }
        }
    }
}

use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct Props {
    message: Element,
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
pub fn Hero() -> Element {
    let mut crate_name = use_signal(|| "cellular-raza".to_string());
    let mut version = use_signal(|| "0.1".to_string());
    let mut messages = use_signal(|| vec![]);

    let mut update_form = move |event: Event<FormData>| async move {
        let default = vec!["__nothing__".to_string()];
        let values: std::collections::HashMap<_, _> = event
            .data
            .values()
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();
        let cn = &values.get("crate_name").unwrap_or(&default)[0];
        let ve = &values.get("version").unwrap_or(&default)[0];
        crate_name.set(format!("{cn}"));
        version.set(format!("{ve}"));
        match crate2bib::get_biblatex(&crate_name.to_string(), &version.to_string(), None).await {
            // TODO rework this; how can we display multiple results?
            Ok(results) => {
                for (entry, origin) in results {
                    let (name, link) = match origin {
                        crate2bib::EntryOrigin::CratesIO => (
                            "crates.io".to_string(),
                            format!("https://crates.io/crates/{crate_name}"),
                        ),
                        crate2bib::EntryOrigin::CitationCff => (
                            "CITATION.cff".to_string(),
                            entry.url.clone().unwrap_or_default(),
                        ),
                    };
                    let height = format!("{entry}").lines().count() + 5;
                    messages.push(Success(Props {
                        message: rsx! {
                            p {
                                "Found entry for "
                                code { {crate_name} }
                                " from "
                                a { href: {link}, {name} }
                            }
                            textarea {
                                class: "response",
                                height: "{height}em",
                                "{entry}",
                            }
                        },
                    }));
                }
            }
            Err(e) => {
                messages.push(Error(Props {
                    message: rsx! { "ERROR: {e}" },
                }));
            }
        }
    };

    rsx! {
        div { id: "hero",
            h1 { "crate2Bib" }
            h3 { "Create a BibLaTeX entry from a given crate and version number." }
            form { onsubmit: move |event| update_form(event),
                input { name: "crate_name", r#type: "text", value: crate_name }
                input { name: "version", r#type: "text", value: version }
                input { value: "Generate", r#type: "submit" }
            }
            h2 { "BibLaTeX Citation" }
            p {
                "The "
                a { href: "https://github.com/jonaspleyer/crate2bib", "crate2bib" }
                " crate scans "
                a { href: "https://crates.io/", "crates.io" }
                " for possible candidates and then searches for any "
                code { "CITATION.cff" }
                " files inside the respective repository of the candidate."
            }

            for i in 0..messages.len() {
                div { style: "margin: 0.5em;",
                    {&messages.get(messages.len() - i - 1).unwrap().clone()}
                }
            }
        }
    }
}

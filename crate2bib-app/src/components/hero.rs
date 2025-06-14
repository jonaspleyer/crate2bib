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
    let mut messages = use_signal(|| circ_buffer::RingBuffer::<_, 8>::new());

    let update_form = move |event: Event<FormData>| async move {
        let values: std::collections::HashMap<_, _> = event
            .data
            .values()
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();
        let crate_name = &values.get("crate_name").unwrap()[0];
        let version: Option<&String> = values.get("version").and_then(|x| x.first());
        let mut y = String::new();
        match crate2bib::get_biblatex(
            crate_name,
            version.and_then(|x| {
                y = x.replace(" ", "");
                if y.is_empty() {
                    None
                } else {
                    Some(y.as_str())
                }
            }),
            None,
            None,
            vec!["CITATION.cff", "citation.bib"],
        )
        .await
        {
            // TODO rework this; how can we display multiple results?
            Ok(results) => {
                for entry in results.into_iter().rev() {
                    let (name, link, found_message) = match entry {
                        crate2bib::BibLaTeX::CratesIO(ref e) => (
                            "crates.io".to_string(),
                            format!("https://crates.io/crates/{crate_name}"),
                            // e.version.as_ref().map(|x| format!("{x}")),
                            if let Some(v) = &e.version {
                                format!("{crate_name} {}", v)
                            } else {
                                crate_name.clone()
                            },
                        ),
                        crate2bib::BibLaTeX::CITATIONCFF(ref e) => (
                            "CITATION.cff".to_string(),
                            e.url.clone().map_or("".to_string(), |x| format!("{x}")),
                            // e.version.clone(),
                            if let Some(v) = &e.version {
                                format!("{crate_name} {}", v)
                            } else {
                                crate_name.clone()
                            },
                        ),
                        #[allow(unused)]
                        crate2bib::BibLaTeX::Plain(crate2bib::PlainBibLaTeX {
                            ref bibliography,
                            ref repository,
                            ref filename,
                        }) => (
                            "bibliography file".to_string(),
                            repository.clone(),
                            // None,
                            crate_name.clone(),
                        ),
                    };
                    let height = format!("{entry}").lines().count() + 5;
                    messages.write().push(Success(Props {
                        message: rsx! {
                            p {
                                "Found entry for "
                                code { "{found_message})" }
                                " from "
                                a { href: link, {name} }
                            }
                            textarea { class: "response", height: "{height}em", "{entry}" }
                        },
                    }));
                }
            }
            Err(e) => {
                messages.write().push(Error(Props {
                    message: rsx! { "ERROR: {e}" },
                }));
            }
        }
    };

    rsx! {
        div { id: "hero", class: "middle",
            h1 { "crate2Bib" }
            h3 { "Create a BibLaTeX entry from a given crate and version number." }
            form { onsubmit: move |event| update_form(event),
                input {
                    name: "crate_name",
                    r#type: "text",
                    value: "cellular-raza",
                }
                input { name: "version", r#type: "text", value: "0.2" }
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

            for i in 0..messages.read().len() {
                div { style: "margin: 0.5em;",
                    {&messages.read()[messages.read().len() - i - 1].clone()}
                }
            }
        }
        footer {
            div { class: "middle",
                a {
                    class: "nav-item",
                    href: "https://github.com/jonaspleyer/crate2bib",
                    img { src: crate::GITHUB_MARK_WHITE },
                }
            }
        }
    }
}

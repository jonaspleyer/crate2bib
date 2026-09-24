use crate2bib::BibReturn;
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

fn convert_entry(entry: crate2bib::BibLaTeX, crate_name: &str) -> Element {
    let (name, link, found_message) = match entry {
        crate2bib::BibLaTeX::CratesIO(ref e) => (
            "crates.io".to_string(),
            format!("https://crates.io/crates/{crate_name}"),
            // e.version.as_ref().map(|x| format!("{x}")),
            if let Some(v) = &e.version {
                format!("{crate_name} {}", v)
            } else {
                crate_name.to_string()
            },
        ),
        crate2bib::BibLaTeX::CITATIONCFF(ref e) => (
            "CITATION.cff".to_string(),
            e.url.clone().map_or("".to_string(), |x| format!("{x}")),
            // e.version.clone(),
            if let Some(v) = &e.version {
                format!("{crate_name} {}", v)
            } else {
                crate_name.to_string()
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
            crate_name.to_string(),
        ),
    };
    Success(Props {
        message: rsx! {
            p {
                "Found entry for "
                code { "{found_message})" }
                " from "
                a { href: link, {name} }
            }
            textarea { class: "response", "{entry}" }
        },
    })
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
        let search_type = &values.get("search_type").unwrap();
        let full = &values.get("input_text").unwrap();
        if search_type == &"DOI" {
            match crate2bib::get_bibtex_doi_without_client(full, None).await {
                Ok(bib) => {
                    let bib_output = match bib {
                        BibReturn::BibFile(bib) => bib.to_biblatex_string(),
                        BibReturn::String(s) => s,
                    }
                    .split_once("\n")
                    .map(|(first, rest)| format!("{}\n{rest}", first.replace("_", "")))
                    .unwrap_or_default()
                    .replace(",\n", ",\n    ")
                    .replace(",\n    }", ",\n}")
                    .trim_end()
                    .to_string();
                    messages.write().push(Success(Props {
                        message: rsx! {
                            p {
                                "Found DOI "
                                a { href: "https://doi.org/{full}", code { "{full}" } }
                            }
                            textarea { class: "response", "{bib_output}" }
                        },
                    }));
                }
                Err(e) => messages.write().push(Error(Props {
                    message: rsx! { "ERROR: {e}"},
                })),
            }
            return;
        };

        let (crate_name, version): (&str, Option<&str>) = if full.contains("@") {
            let mut split = full.split("@");
            let crate_name = split.next().unwrap_or_default();
            let version = split.next();
            (crate_name, version)
        } else {
            (full, None)
        };
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
            Ok(results) => {
                for entry in results.into_iter().rev() {
                    match entry {
                        Ok(entry) => messages.write().push(convert_entry(entry, crate_name)),
                        Err(e) => messages.write().push(Error(Props {
                            message: rsx! { "ERROR: {e}" },
                        })),
                    }
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
            h3 { "Create a BibLaTeX entry from a given crate or DOI" }
            form { onsubmit: update_form,
                input {
                    name: "input_text",
                    r#type: "text",
                    value: "cellular-raza@0.4",
                }
                select { name: "search_type",
                    option { value: "crate", "crate" }
                    option { value: "DOI", "DOI" }
                }
                input { value: "search", r#type: "submit" }
            }
            h2 { "BibLaTeX Citation" }

            for i in 0..messages.read().len() {
                div { style: "margin: 0.5em;",
                    {&messages.read()[messages.read().len() - i - 1].clone()}
                }
            }

            p {
                a { href: "https://github.com/jonaspleyer/crate2bib", "crate2bib" }
                " scans "
                a { href: "https://crates.io/", "crates.io" }
                " for possible candidates and then searches for any "
                code { "CITATION.cff" }
                " files inside the respective repository of the candidate."
            }
            p {
                "BibTeX entries for DOIs are obtained directly from "
                a { href: "https://doi.org/", "doi.org" }
            }
        }
    }
}

use dioxus::prelude::*;

const HEADER_SVG: Asset = asset!("/assets/header.svg");

#[component]
pub fn Hero() -> Element {
    let mut crate_name = use_signal(|| "cellular-raza".to_string());
    let mut version = use_signal(|| "0.1".to_string());
    let mut bibtex = use_signal(|| "__empty__".to_string());

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
        let bib = crate2bib::get_bibtex(&crate_name.to_string(), &version.to_string()).await;
        match bib {
            Ok(v) => {
                bibtex.set(format!("{v}"));
                messages.push(Success(Props {
                    message: format!("SUCCESS: {crate_name}"),
                }));
            }
            Err(e) => {
                messages.push(Error(Props {
                    message: format!("ERROR: {e}"),
                }));
            }
        }
    };

    rsx! {
        div { id: "hero",
            h1 { "crate2Bib" }
            h3 { "Create a BibTeX entry from a given crate and version number." }
            form { onsubmit: move |event| update_form(event),
                input { name: "crate_name", r#type: "text", value: crate_name }
                input { name: "version", r#type: "text", value: version }
                input { value: "Generate", r#type: "submit" }
            }
            h2 { "BibTeX Citation" }
            textarea { id: "response", value: bibtex }
            p {
                "The "
                a { href: "https://github.com/jonaspleyer/crate2bib", "crate2bib" }
                " crate scans "
                a { href: "https://crates.io/", "crates.io" }
                " for possible candidates and then searches for any "
                code { "CITATION.cff" }
                " files inside the respective repository of the candidate."
            }
            }
        }
    }
}

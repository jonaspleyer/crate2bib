//! Search and create BibLaTeX entries for crates hosted on [crates.io](https://crates.io)
//! or retrieve them from their github repository.
//!
//! This crate can be used in a web version under
//! [jonaspleyer.github.io/crate2bib](https://jonaspleyer.github.io/crate2bib).
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod crates_io;
mod doi;
mod github;
#[cfg_attr(docsrs, doc(cfg(feature = "pyo3")))]
#[cfg(feature = "pyo3")]
mod python;
mod types;

pub use crates_io::*;
pub use doi::*;
pub use github::*;
pub use types::*;

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn obtain_from_doi_org() {
        let expected = r#"@article{Pleyer_2025,
author = {Pleyer, Jonas and Fleck, Christian},
doi = {10.21105/joss.07723},
issn = {2475-9066},
journaltitle = {Journal of Open Source Software},
month = {June},
number = {110},
pages = {7723},
publisher = {The Open Journal},
title = {cellular\_raza: Cellular Agent-based Modeling from a Clean Slate},
url = {http://dx.doi.org/10.21105/joss.07723},
volume = {10},
year = {2025},
}
"#;

        let results = get_biblatex(
            "cellular_raza",
            None,
            Some("asdf"),
            None,
            vec!["CITATION.cff"],
        )
        .await
        .unwrap();

        for r in results.iter() {
            if let BibLaTeX::Plain(_) = r {
                assert_eq!(expected, format!("{r}"));
            }
        }
    }
}

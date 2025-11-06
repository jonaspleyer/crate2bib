use dioxus::prelude::*;

mod main;

const PYPI_LOGO: Asset = asset!("/assets/pypi-logo.svg");
const RUST_LOGO: Asset = asset!("/assets/rust-logo-white.svg");

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer {
            div { class: "middle",
                a {
                    class: "nav-item",
                    href: "https://pypi.org/project/crate2bib/",
                    img { src: PYPI_LOGO }
                    "Python Docs"
                }
                a { class: "nav-item", href: "https://docs.rs/crate2bib",
                    img { src: RUST_LOGO }
                    "Rust Docs"
                }
                a {
                    class: "nav-item",
                    href: "https://github.com/jonaspleyer/crate2bib",
                    img { src: crate::GITHUB_MARK_WHITE }
                    "Github"
                }
            }
        }
    }
}

pub use main::Main;

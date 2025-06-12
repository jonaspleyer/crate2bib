use dioxus::prelude::*;

use components::Hero;

mod components;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const GITHUB_MARK_WHITE: Asset = asset!("/assets/github-mark-white.svg");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const ADMONITION_CSS: Asset = asset!("/assets/styling/admonitions.css");

#[derive(Debug)]
pub(crate) enum ColorMode {
    Light,
    Dark,
}

fn main() {
    if tracing_log::LogTracer::init().is_err() {
        println!("Could not set up logger");
    }
    launch(App);
}

#[derive(Routable, Clone)]
enum Route {
    #[route("/")]
    Hero {},
}

#[component]
fn App() -> Element {
    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: ADMONITION_CSS }

        Router::<Route> {}
    }
}

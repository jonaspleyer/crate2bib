pub use wasm_bindgen::prelude::*;

// pub type Result<T> = core::result::Result<T, wasm_bindgen::JsError>;

#[wasm_bindgen]
pub struct BibLatex {}

impl core::fmt::Display for BibLatex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO
        f.write_str("")
    }
}

#[no_mangle]
#[wasm_bindgen]
pub async fn create_bib(input: &str) -> Result<BibLatex, JsValue> {
    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(web_sys::RequestMode::Cors);
    let url = format!("https://github.com/jonaspleyer/cellular_raza/blob/master/CITATION.cff");
    let request = web_sys::Request::new_with_str_and_init(&url, &opts)?;
    request
        .headers()
        .set("Accept", "application/vnd.github.v3+json")?;
    web_sys::console::log_1(&request);
    let window = web_sys::window().unwrap();
    let resp_value =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
    web_sys::console::log_1(&resp_value);
    Ok(BibLatex {})
}

#[no_mangle]
#[wasm_bindgen]
pub async fn create_bib_string(input: String) -> Result<String, JsValue> {
    let b: BibLatex = create_bib(&input).await?;
    Ok(format!("{b}"))
}

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    // let body = document.body().expect("document should have a body");
    // let main = document.get_element_by_id("main").expect("no main div found");
    if let (Some(_), Some(_), Some(input)) = (
        document.get_element_by_id("results"),
        document.get_element_by_id("crate_name"),
        document.get_element_by_id("create-citation-button"),
    ) {
        input.set_attribute(
            "onclick",
            "results.innerText = create_bib_string(crate_name.value)",
        )?;
    } else {
        web_sys::console::log_1(&"Could not find html element".into());
    }
    Ok(())
}

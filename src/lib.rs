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
pub async fn create_bib(input: &str) -> Result<BibLatex, JsError> {
    let body = reqwest::get(format!("https://crates.io/crates/{input}"))
        .await?
        .text()
        .await?;
    println!("{}", body);
    Ok(BibLatex {})
}

#[no_mangle]
#[wasm_bindgen]
pub async fn create_bib_string(input: String) -> Result<String, JsError> {
    let b: BibLatex = create_bib(&input).await?;
    Ok(format!("{b}"))
}

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    /* let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let val = document.create_element("p")?;
    val.set_inner_html("Hello from Rust!");
    body.append_child(&val)?;*/
    Ok(())
}

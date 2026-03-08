mod jwt_decoder;
mod password_gen;
mod utils;

use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, Window};

fn window() -> Window {
    web_sys::window().expect("no global window")
}

fn document() -> Document {
    window().document().expect("no document on window")
}

fn get_hash() -> String {
    window()
        .location()
        .hash()
        .unwrap_or_default()
        .trim_start_matches('#')
        .to_string()
}

fn app_root() -> Element {
    document()
        .get_element_by_id("app")
        .expect("no #app element")
}

struct UtilEntry {
    id: &'static str,
    name: &'static str,
    description: &'static str,
}

const UTILS: &[UtilEntry] = &[
    UtilEntry {
        id: "password-gen",
        name: "Password Generator",
        description: "Generate deterministic passwords from a seed phrase",
    },
    UtilEntry {
        id: "jwt-decoder",
        name: "JWT Decoder",
        description: "Decode and inspect JSON Web Token headers and payloads",
    },
];

fn render_home(root: &Element) {
    let mut html = String::from(
        r#"<div class="home">
            <h1>Utils</h1>
            <p class="subtitle">A collection of handy browser-based utilities</p>
            <div class="grid">"#,
    );

    for util in UTILS {
        html.push_str(&format!(
            r##"<a class="card" href="#{id}">
                <h2>{name}</h2>
                <p>{desc}</p>
            </a>"##,
            id = util.id,
            name = util.name,
            desc = util.description,
        ));
    }

    html.push_str("</div></div>");
    root.set_inner_html(&html);
}

fn route() {
    let root = app_root();
    let hash = get_hash();

    match hash.as_str() {
        "password-gen" => password_gen::render(&root),
        "jwt-decoder" => jwt_decoder::render(&root),
        _ => render_home(&root),
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    utils::set_panic_hook();
    route();

    let cb = Closure::<dyn Fn()>::new(|| {
        route();
    });

    window().set_onhashchange(Some(cb.as_ref().unchecked_ref()));
    cb.forget();

    Ok(())
}

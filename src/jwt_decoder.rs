use crate::document;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use wasm_bindgen::prelude::*;
use web_sys::Element;

// --- Constants ---

const JWT_PART_COUNT: usize = 3;
const JWT_DELIMITER: char = '.';
const JSON_INDENT_SPACES: u32 = 2;

const EL_INPUT: &str = "jwt-input";
const EL_HIGHLIGHT: &str = "jwt-highlight";
const EL_ERROR: &str = "jwt-error";
const EL_HEADER: &str = "jwt-header";
const EL_PAYLOAD: &str = "jwt-payload";

const CSS_HEADER: &str = "jwt-part-header";
const CSS_PAYLOAD: &str = "jwt-part-payload";
const CSS_SIGNATURE: &str = "jwt-part-signature";
const CSS_DELIMITER: &str = "jwt-part-dot";

#[derive(Debug, Clone, Copy, PartialEq)]
enum JwtPart {
    Header,
    Payload,
}

#[derive(Debug, PartialEq)]
enum DecodeError {
    InvalidStructure,
    InvalidBase64(JwtPart),
    InvalidUtf8(JwtPart),
    InvalidJson(JwtPart),
}

impl DecodeError {
    fn message(&self) -> &str {
        match self {
            DecodeError::InvalidStructure => "JWT must have exactly 3 dot-separated parts",
            DecodeError::InvalidBase64(JwtPart::Header) => {
                "Header contains invalid base64url characters"
            }
            DecodeError::InvalidBase64(JwtPart::Payload) => {
                "Payload contains invalid base64url characters"
            }
            DecodeError::InvalidUtf8(JwtPart::Header) => "Header is not valid UTF-8",
            DecodeError::InvalidUtf8(JwtPart::Payload) => "Payload is not valid UTF-8",
            DecodeError::InvalidJson(JwtPart::Header) => "Header is not valid JSON",
            DecodeError::InvalidJson(JwtPart::Payload) => "Payload is not valid JSON",
        }
    }
}

#[derive(Debug)]
struct DecodedJwt {
    header: String,
    payload: String,
}

fn decode_jwt(token: &str) -> Result<DecodedJwt, DecodeError> {
    let token = token.trim();
    let parts: Vec<&str> = token.split(JWT_DELIMITER).collect();

    if parts.len() != JWT_PART_COUNT {
        return Err(DecodeError::InvalidStructure);
    }

    let header = decode_part(parts[0], JwtPart::Header)?;
    let payload = decode_part(parts[1], JwtPart::Payload)?;

    Ok(DecodedJwt { header, payload })
}

fn decode_part(encoded: &str, part: JwtPart) -> Result<String, DecodeError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| DecodeError::InvalidBase64(part))?;
    let json_str = String::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8(part))?;

    // Validate and pretty-print JSON via js_sys
    let js_val = js_sys::JSON::parse(&json_str).map_err(|_| DecodeError::InvalidJson(part))?;
    let pretty = js_sys::JSON::stringify_with_replacer_and_space(
        &js_val,
        &JsValue::NULL,
        &JSON_INDENT_SPACES.into(),
    )
    .map_err(|_| DecodeError::InvalidJson(part))?;

    Ok(pretty.as_string().unwrap_or(json_str))
}

// --- DOM rendering ---

fn set_element_text(id: &str, text: &str) {
    if let Some(el) = document().get_element_by_id(id) {
        el.set_text_content(Some(text));
    }
}

fn set_element_html(id: &str, html: &str) {
    if let Some(el) = document().get_element_by_id(id) {
        el.set_inner_html(html);
    }
}

fn show_error(msg: &str) {
    if let Some(el) = document().get_element_by_id(EL_ERROR) {
        el.set_text_content(Some(msg));
        el.set_attribute("style", "display: block").ok();
    }
}

fn clear_error() {
    if let Some(el) = document().get_element_by_id(EL_ERROR) {
        el.set_text_content(None);
        el.set_attribute("style", "display: none").ok();
    }
}

fn clear_output() {
    set_element_text(EL_HEADER, "");
    set_element_text(EL_PAYLOAD, "");
}

fn get_input_value(id: &str) -> String {
    document()
        .get_element_by_id(id)
        .and_then(|el| el.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

fn colorize_token(input: &str) -> String {
    let parts: Vec<&str> = input.split(JWT_DELIMITER).collect();
    if parts.len() != JWT_PART_COUNT {
        return html_escape(input);
    }

    let classes = [CSS_HEADER, CSS_PAYLOAD, CSS_SIGNATURE];
    let mut result = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            result.push_str(&format!("<span class=\"{CSS_DELIMITER}\">.</span>"));
        }
        result.push_str(&format!(
            "<span class=\"{}\">{}</span>",
            classes[i],
            html_escape(part)
        ));
    }
    result
}

fn update_output() {
    let token = get_input_value(EL_INPUT);

    // Always update the highlight overlay
    set_element_html(EL_HIGHLIGHT, &colorize_token(&token));

    if token.trim().is_empty() {
        clear_error();
        clear_output();
        return;
    }

    match decode_jwt(&token) {
        Ok(decoded) => {
            clear_error();
            set_element_html(
                EL_HEADER,
                &format!("<pre>{}</pre>", html_escape(&decoded.header)),
            );
            set_element_html(
                EL_PAYLOAD,
                &format!("<pre>{}</pre>", html_escape(&decoded.payload)),
            );
        }
        Err(e) => {
            show_error(e.message());
            clear_output();
        }
    }
}

fn sync_scroll() {
    let input = document()
        .get_element_by_id(EL_INPUT)
        .and_then(|el| el.dyn_into::<web_sys::HtmlTextAreaElement>().ok());
    let highlight = document().get_element_by_id(EL_HIGHLIGHT);

    if let (Some(input), Some(highlight)) = (input, highlight) {
        highlight.set_scroll_top(input.scroll_top());
        highlight.set_scroll_left(input.scroll_left());
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn render(root: &Element) {
    root.set_inner_html(
        r##"<div class="util-page">
            <a class="back" href="#">&larr; Back</a>
            <h1>JWT Decoder</h1>
            <p class="subtitle">Paste a JSON Web Token to decode its header and payload</p>

            <div class="form-group">
                <label for="jwt-input">Token</label>
                <div class="jwt-input-wrap">
                    <div id="jwt-highlight" class="jwt-highlight" aria-hidden="true"></div>
                    <textarea id="jwt-input" rows="5" placeholder="Paste your JWT here..." spellcheck="false"></textarea>
                </div>
            </div>

            <div id="jwt-error" class="error-message" style="display: none"></div>

            <div class="jwt-results">
                <div class="form-group">
                    <label>Header</label>
                    <div id="jwt-header" class="jwt-output"></div>
                </div>

                <div class="form-group">
                    <label>Payload</label>
                    <div id="jwt-payload" class="jwt-output"></div>
                </div>
            </div>
        </div>"##,
    );

    let input_cb = Closure::<dyn Fn()>::new(|| update_output());
    let scroll_cb = Closure::<dyn Fn()>::new(|| sync_scroll());
    if let Some(el) = document().get_element_by_id(EL_INPUT) {
        el.add_event_listener_with_callback("input", input_cb.as_ref().unchecked_ref())
            .ok();
        el.add_event_listener_with_callback("scroll", scroll_cb.as_ref().unchecked_ref())
            .ok();
    }
    input_cb.forget();
    scroll_cb.forget();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_parts() {
        assert_eq!(
            decode_jwt("only.two").unwrap_err(),
            DecodeError::InvalidStructure
        );
    }

    #[test]
    fn rejects_too_many_parts() {
        assert_eq!(
            decode_jwt("a.b.c.d").unwrap_err(),
            DecodeError::InvalidStructure
        );
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(decode_jwt("").unwrap_err(), DecodeError::InvalidStructure);
    }

    #[test]
    fn rejects_invalid_base64_header() {
        assert!(matches!(
            decode_jwt("!!!.valid.sig"),
            Err(DecodeError::InvalidBase64(JwtPart::Header))
        ));
    }

    #[test]
    fn rejects_single_part() {
        assert_eq!(
            decode_jwt("onlyonepart").unwrap_err(),
            DecodeError::InvalidStructure
        );
    }
}

use wasm_bindgen::prelude::*;
use web_sys::Element;

use crate::document;

const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &[u8] = b"0123456789";
const SPECIAL: &[u8] = b"!@#$%^&*()-_=+[]{}|;:,.<>?";

/// Simple deterministic hash: produces a sequence of bytes from a seed string.
/// Not cryptographically secure — intended for repeatable password generation.
fn hash_seed(seed: &str, length: usize) -> Vec<u8> {
    let seed_bytes = seed.as_bytes();
    if seed_bytes.is_empty() {
        return vec![0; length];
    }

    let mut state: [u64; 4] = [
        0x6a09_e667_f3bc_c908,
        0xbb67_ae85_84ca_a73b,
        0x3c6e_f372_fe94_f82b,
        0xa54f_f53a_5f1d_36f1,
    ];

    // Mix seed into state
    for (i, &b) in seed_bytes.iter().enumerate() {
        let idx = i % 4;
        state[idx] = state[idx]
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(b as u64)
            .wrapping_add(i as u64);
        // Cross-mix
        state[(idx + 1) % 4] ^= state[idx];
    }

    // Additional mixing rounds
    for round in 0..8 {
        for idx in 0..4 {
            state[idx] = state[idx]
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(round * 4 + idx as u64);
            state[(idx + 1) % 4] ^= state[idx] >> 17;
        }
    }

    // Generate output bytes
    let mut result = Vec::with_capacity(length);
    let mut counter: u64 = 0;
    while result.len() < length {
        let mix = state[(counter as usize) % 4]
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(counter);
        counter += 1;
        state[(counter as usize) % 4] = state[(counter as usize) % 4].wrapping_add(mix >> 13);

        for &byte in &mix.to_le_bytes() {
            if result.len() < length {
                result.push(byte);
            }
        }
    }

    result
}

fn generate_password(seed: &str, length: usize, use_numbers: bool, use_special: bool) -> String {
    if seed.is_empty() {
        return String::new();
    }

    let mut charset = Vec::new();
    charset.extend_from_slice(LOWERCASE);
    charset.extend_from_slice(UPPERCASE);
    if use_numbers {
        charset.extend_from_slice(DIGITS);
    }
    if use_special {
        charset.extend_from_slice(SPECIAL);
    }

    let hash = hash_seed(seed, length);
    hash.iter()
        .map(|&b| charset[(b as usize) % charset.len()] as char)
        .collect()
}

fn get_input_value(id: &str) -> String {
    document()
        .get_element_by_id(id)
        .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

fn is_checked(id: &str) -> bool {
    document()
        .get_element_by_id(id)
        .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.checked())
        .unwrap_or(false)
}

fn update_password() {
    let seed = get_input_value("seed-input");
    let length: usize = get_input_value("length-slider")
        .parse()
        .unwrap_or(16);
    let use_numbers = is_checked("use-numbers");
    let use_special = is_checked("use-special");

    let password = generate_password(&seed, length, use_numbers, use_special);

    if let Some(el) = document().get_element_by_id("password-output") {
        el.set_text_content(Some(&password));
    }
    if let Some(el) = document().get_element_by_id("length-value") {
        el.set_text_content(Some(&length.to_string()));
    }
}

fn attach_listener(id: &str, event: &str) {
    let cb = Closure::<dyn Fn()>::new(|| update_password());
    if let Some(el) = document().get_element_by_id(id) {
        el.add_event_listener_with_callback(event, cb.as_ref().unchecked_ref())
            .ok();
    }
    cb.forget();
}

pub fn render(root: &Element) {
    root.set_inner_html(
        r##"<div class="util-page">
            <a class="back" href="#">&larr; Back</a>
            <h1>Password Generator</h1>
            <p class="subtitle">Generate a deterministic password from a seed phrase</p>

            <div class="form-group">
                <label for="seed-input">Seed</label>
                <input type="text" id="seed-input" placeholder="Enter your seed phrase..." autocomplete="off" />
            </div>

            <div class="form-group">
                <label for="length-slider">Length: <span id="length-value">16</span></label>
                <input type="range" id="length-slider" min="8" max="64" value="16" />
            </div>

            <div class="form-group checkbox-group">
                <label><input type="checkbox" id="use-numbers" checked /> Include numbers</label>
                <label><input type="checkbox" id="use-special" checked /> Include special characters</label>
            </div>

            <div class="form-group">
                <label>Generated Password</label>
                <div class="output-row">
                    <code id="password-output" class="password-display"></code>
                    <button id="copy-btn" class="btn" title="Copy to clipboard">Copy</button>
                </div>
            </div>
        </div>"##,
    );

    // Attach event listeners
    attach_listener("seed-input", "input");
    attach_listener("length-slider", "input");
    attach_listener("use-numbers", "change");
    attach_listener("use-special", "change");

    // Copy button
    let copy_cb = Closure::<dyn Fn()>::new(|| {
        let text = document()
            .get_element_by_id("password-output")
            .and_then(|el| el.text_content())
            .unwrap_or_default();

        if text.is_empty() {
            return;
        }

        let win = crate::window();
        let nav = js_sys::Reflect::get(&win, &"navigator".into()).unwrap();
        let clip = js_sys::Reflect::get(&nav, &"clipboard".into()).unwrap();
        let write_text = js_sys::Reflect::get(&clip, &"writeText".into()).unwrap();
        let _ = js_sys::Reflect::apply(
            &js_sys::Function::from(write_text),
            &clip,
            &js_sys::Array::of1(&text.into()),
        );
    });
    if let Some(el) = document().get_element_by_id("copy-btn") {
        el.add_event_listener_with_callback("click", copy_cb.as_ref().unchecked_ref())
            .ok();
    }
    copy_cb.forget();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_output() {
        let a = generate_password("test-seed", 16, true, true);
        let b = generate_password("test-seed", 16, true, true);
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_differ() {
        let a = generate_password("seed-one", 16, true, true);
        let b = generate_password("seed-two", 16, true, true);
        assert_ne!(a, b);
    }

    #[test]
    fn respects_length() {
        let pw = generate_password("test", 32, true, true);
        assert_eq!(pw.len(), 32);
    }

    #[test]
    fn empty_seed_empty_output() {
        let pw = generate_password("", 16, true, true);
        assert!(pw.is_empty());
    }

    #[test]
    fn no_digits_when_disabled() {
        let pw = generate_password("test-seed-long", 100, false, false);
        assert!(!pw.chars().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn contains_variety() {
        let pw = generate_password("variety-test-seed", 64, true, true);
        assert!(pw.chars().any(|c| c.is_ascii_lowercase()));
        assert!(pw.chars().any(|c| c.is_ascii_uppercase()));
    }
}

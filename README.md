# Utils

A collection of browser-based utilities built with Rust and WebAssembly.

## Available Utils

- **Password Generator** - Generate deterministic passwords from a seed phrase with configurable length, numbers, and special characters.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- A local HTTP server (e.g., `python -m http.server`, [miniserve](https://github.com/svenstaro/miniserve), or similar)

Install the wasm32 target and wasm-pack:

```sh
rustup target add wasm32-unknown-unknown
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

## Local Development

Build the WASM package:

```sh
wasm-pack build --target web --out-dir www/pkg
```

Serve the `www/` directory:

```sh
# Using Python
python -m http.server -d www 8080

# Or using miniserve
miniserve www --index index.html -p 8080
```

Then open `http://localhost:8080` in your browser.

## Running Tests

```sh
cargo test
```

## Adding a New Util

1. Create a new module in `src/` (e.g., `src/my_util.rs`)
2. Add `mod my_util;` to `src/lib.rs`
3. Add an entry to the `UTILS` array in `src/lib.rs`
4. Add a match arm in the `route()` function
5. Implement a `pub fn render(root: &Element)` in your module

## Deployment

Pushes to `main` automatically build and deploy to GitHub Pages via the workflow in `.github/workflows/deploy.yml`. To enable this, go to your repository settings and set GitHub Pages source to **GitHub Actions**.

## Project Structure

```
src/
  lib.rs           # Router and home page
  password_gen.rs  # Password generator util
  utils.rs         # Shared helpers (panic hook)
www/
  index.html       # HTML shell
  style.css        # Styles
tests/
  web.rs           # WASM integration tests
```

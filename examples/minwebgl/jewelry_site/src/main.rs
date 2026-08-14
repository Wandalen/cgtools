//! This site is 100% static HTML/CSS/JS ( see `index.html`, `index.js`, `index.css` ).
//! `Cargo.toml` exists only so `action/run`'s tag-based tooling can register this as a
//! `runtime:browser` example. `trunk build` always runs `wasm-bindgen` over whatever it
//! compiles here, and `wasm-bindgen-cli` fails to process a binary with zero `wasm-bindgen`
//! crate usage, so this crate exports one inert function purely to give it something valid
//! to process. Nothing in the site loads the generated wasm/js.

fn main() {}

/// Inert export so `wasm-bindgen` has something to process — see module doc.
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn trunk_wasm_bindgen_placeholder() {}

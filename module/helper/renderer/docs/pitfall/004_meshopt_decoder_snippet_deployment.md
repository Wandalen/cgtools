# Pitfall: The meshopt decoder snippet must be deployed

The glTF loader's meshopt decoder is a JS module that wasm-bindgen imports
statically. Every page that links the loader needs it, whether or not its
assets are compressed, and a deployment that drops it, or a build target that
cannot import it, breaks the whole wasm module rather than just compressed
assets.

### Scope

- **Purpose**: Warn every consumer of the glTF loader that it has a deployment requirement beyond the `.js` glue and `.wasm` file.
- **Responsibility**: Document the trap, the concrete failures, and the mitigation.
- **In Scope**: The vendored `meshopt_decoder.js` snippet, its static import, its CSP need and the wasm-bindgen targets it rules out.
- **Out of Scope**: How compressed buffer views are decoded (see [../feature/001_pbr_rendering_core.md](../feature/001_pbr_rendering_core.md) and `src/webgl/loaders/meshopt.rs`).

### Trap

`src/webgl/loaders/meshopt.rs` binds meshoptimizer's decoder with
`#[ wasm_bindgen( module = "/src/webgl/loaders/meshopt_decoder.js" ) ]`.
wasm-bindgen turns that into a JS snippet: it copies the file to
`pkg/snippets/renderer-<hash>/src/webgl/loaders/meshopt_decoder.js`, and the
generated glue imports it at the top. The import is static, so it does not
depend on what is loaded at run time:

- every page that links the crate's loader fetches the snippet, and the snippet
  compiles its inlined wasm with `WebAssembly.instantiate` as soon as it is
  imported;
- wasm-bindgen's `--target no-modules` output cannot import JS modules at all,
  so a crate built that way stops building.

### Failure

- A pipeline that copies only the glue `.js` and the `.wasm` file, and not
  `snippets/`, fails at module import: the browser cannot resolve the
  snippet, so the whole wasm module fails to start, not only compressed
  assets.
- A page whose Content Security Policy lacks `'wasm-unsafe-eval'` in
  `script-src` blocks the decoder's `WebAssembly.instantiate`. The crate's
  own wasm needs the same permission, so in practice this bites a page that
  grants it some other way.
- A `--target no-modules` build fails in wasm-bindgen with an error about
  local JS snippets.

### Mitigation

- Serve the generated `pkg/snippets/` folder next to the glue file.
  `wasm-pack` and `trunk` output already include it.
- Allow `'wasm-unsafe-eval'` in `script-src`.
- Build with a JS-module target: `web` or `bundler`, the wasm-bindgen
  default, not `no-modules`.

### Features

| File | Relationship |
|------|--------------|
| [../feature/001_pbr_rendering_core.md](../feature/001_pbr_rendering_core.md) | Deployment requirement every consumer of the glTF loader inherits |

### Sources

| File | Relationship |
|------|--------------|
| `readme.md` | "Compressed glTF": the same deployment notes for users |
| `src/webgl/loaders/meshopt.rs` | The static `#[ wasm_bindgen( module = ... ) ]` import of the decoder |
| `src/webgl/loaders/meshopt_decoder.js` | The vendored decoder: two inlined wasm builds, compiled at import |

### Tests

| File | Relationship |
|------|--------------|
| `tests/gltf_meshopt_browser_test.rs` | Runs the snippet in a browser, so a broken import fails the suite |

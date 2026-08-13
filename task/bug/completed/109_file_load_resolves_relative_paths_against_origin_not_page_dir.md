# BUG-109: `mingl::web::file::load` resolves document-relative asset paths against the origin root instead of the current page's own directory, breaking every example deployed under a subpath

- **Severity:** Critical
- **state:** Completed
- **Affects:** Every browser example that fetches a runtime asset via a document-relative path (`"static/..."`, no leading `/`) — at least 10 examples calling `gl::file::load` directly, plus ~20 more going through `renderer`'s glTF loader (`asset_uri_resolve` → `gl::file::load`) with a non-empty model folder
- **Component:** `module/min/mingl` (`src/web/file.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-13
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-13
- **Fixed:** 2026-08-13

## Symptom

Live-browser check (via `browsee`, Chromium, real WebGL2 context) of `examples/minwebgl/text_msdf/` served at its real deployed subpath (`http://localhost:8899/minwebgl/text_msdf/`, mirroring GitHub Pages' `/minwebgl/text_msdf/` layout) renders a solid black canvas — no text — and the console shows:

```
panicked at examples/minwebgl/text_msdf/src/json.rs:63:51:
called `Result::unwrap()` on an `Err` value: Error("expected value", line: 1, column: 1)
Uncaught RuntimeError: unreachable
```

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
python3 -m http.server 8899 --directory _site &
curl -s http://localhost:8899/minwebgl/text_msdf/static/font/Alike-Regular.json | head -c 40   # 200, valid JSON
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8899/static/font/Alike-Regular.json   # 404 — the URL the wasm actually fetches
```
**Expected:** the app fetches `static/font/Alike-Regular.json` relative to its own page (`/minwebgl/text_msdf/static/...`) and parses it.
**Actual (pre-fix):** the app fetches `/static/font/Alike-Regular.json` (site root), gets a 404, and — because `mingl::web::file::load`'s doc contract states HTTP error codes do **not** produce an `Err` (`fetch` resolves `Ok` regardless of status) — passes the 404 response body straight to `serde_json::from_str`, which fails to parse it and `.unwrap()`-panics.

## Impact

**Who is affected:** Every example that loads a co-located runtime asset (font atlas, `.obj`/`.glb` model, texture, Lottie JSON, TMX tilemap) via a document-relative path — this is the *default* way `gl::file::load`'s own doc comment tells callers to use it ("Trunk-built examples in this repo expose assets under `static/` by default, so they pass arguments like `"static/foo.obj"`"). On the real deployed site (`.github/workflows/pages.yml` → `action/build_site`, every example under its own subpath like `/minwebgl/text_msdf/`), every such fetch resolves to the wrong URL and either 404s or picks up an unrelated file that happens to exist at the site root.

**What breaks:** `url_resolve`'s document-relative branch (`file_name` with no leading `/` and no scheme) joins against `origin` (`window.location().origin()` — scheme+host+port *only*, never the path) instead of the current page's own directory. This is only correct when a page is served at the domain root (true for an isolated `trunk serve` of a single example — the workflow this helper was written and tested against) and false for this repo's actual deployment shape, where 52 examples share one domain under per-example subpaths.

Confirmed via direct grep: 10 examples call `gl::file::load` with a relative path directly (`diamond`, `lottie_surface_rendering`, `make_cube_map`, `object_picking`, `obj_load`, `obj_viewer`, `text_msdf`, `text_rendering`, `wfc`, `deffered_rendering`); ~20 more route through `renderer::webgl::loaders::gltf::asset_uri_resolve`, which folds a non-empty model folder into a relative string before handing it to `gl::file::load` (`animation_amplitude_change`, `animation_surface_rendering`, `area_light`, `character_control`, `curve_surface_rendering`, `deferred_shading`, `gltf_viewer`, `morph_targets`, `narrow_outline`, `outline`, `pbr_lighting`, `postprocessing`, `renderer_with_outlines`, `shadowmap`, `skeletal_animation`, and others).

## How Discovered

The user asked "does the site work now? did you check each example?" after an earlier verification pass had only confirmed `trunk build`/`cargo build`/`verb/test` (compile + lint + unit/doctest + wasm32-compile-check + a handful of `wasm_bindgen_test` suites) — none of which execute a page in a real browser under its actual deployed subpath. Loading `text_msdf` in a real Chromium session via `browsee`, served from `_site/` (the exact `action/build_site` staging output the Pages workflow deploys), surfaced the panic above.

## Fix

`module/min/mingl/src/web/file.rs`:
- `url_resolve` now takes the current document's full URL (`base_href`, i.e. `window.location().href()`) instead of just `origin`, and resolves document-relative `file_name`s against `base_href`'s own directory (path truncated after its final `/`) rather than the origin root. Origin-absolute (leading `/`) and self-contained (`http(s)://`, `//`, `blob:`, `data:`) forms are unchanged.
- `load()` now passes `window.location().href()` instead of `window.location().origin()`.
- Doc comments on both functions updated to describe the corrected contract.

`module/min/mingl/tests/tests/web_file_test.rs`: rewrote the document-relative test cases to use a `base_href` that includes a subpath (matching this repo's real deployment shape) and assert resolution against that subpath's directory, not the origin root — this is the regression coverage for this exact bug.

3-field comment (`Fix(BUG-109)` / `Root cause` / `Pitfall`) at the fix site in `file.rs`.

**Verification:** `cargo test -p mingl` — rewritten pure-function regression tests pass (RED confirmed pre-fix by reverting the join target, GREEN post-fix). Live-browser re-check (`browsee`, Chromium, real WebGL2) of `text_msdf` plus 5 more glTF-loader-based examples (`gltf_viewer`, `skeletal_animation`, `pbr_lighting`, `deferred_shading`, `character_control` — 6/6 sampled) served from `_site/` under their real deployed subpaths — all render correctly, zero relative-path 404s. Full-workspace `verb/test` re-run: nextest 1690/1690, doctests pass, clippy clean, wasm32 check 49/49 examples (3 initially-failed examples traced to an unrelated known cause — a repo-root temp-file sweeper corrupting `target/` mid-build — confirmed passing clean on narrow-scope retry), wasm32 test 3/3 crates — no regressions from this fix. The remaining ~24 affected examples were not individually browser-checked; they share the identical `url_resolve`/`gl::file::load` code path and are covered by the wasm32 compile-check, not a live-render spot check.

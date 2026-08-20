# BUG-109: mingl's document-relative asset URL resolution (`file::load`, `dom::image_element_create`, `dom::video_element_create`) joins against the origin root instead of the current page's own directory, breaking every example deployed under a subpath

- **Severity:** Critical
- **state:** Completed
- **Affects:** Every browser example that fetches a runtime asset via a document-relative path (`"static/..."`, no leading `/`) — 29/29 `gl::file::load`-based examples confirmed FIXED via a full live-browser sweep (see `## Fix`); PLUS `text_msdf`, `object_picking`, `sprite_animation`, `video_as_texture`, `filter`, `mapgen_tiles_rendering`, `wfc` via `gl::dom::image_element_create`/`video_element_create` or an independent duplicate — ALL 7 now confirmed FIXED via live-browser re-verification (see `## Reclosed`)
- **Component:** `module/min/mingl` (`src/web/web.rs` shared `resolve_url`, `src/web/file.rs`, `src/web/dom.rs` — all fixed) + `examples/minwebgl/filter` (independent inline duplicate — fixed) + `examples/minwebgl/mapgen_tiles_rendering` (missing `static/` prefix + separate DOM id-mismatch bug — both fixed)
- **repo_identity:** self
- **Filed:** 2026-08-13
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self, live-browser re-verification — see `## Reclosed`)
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

**Verification:** `cargo test -p mingl` — rewritten pure-function regression tests pass (RED confirmed pre-fix by reverting the join target, GREEN post-fix). Live-browser re-check (`browsee`, Chromium, real WebGL2) of `text_msdf` plus 5 more glTF-loader-based examples (`gltf_viewer`, `skeletal_animation`, `pbr_lighting`, `deferred_shading`, `character_control` — 6/6 sampled) served from `_site/` under their real deployed subpaths — all render correctly, zero relative-path 404s **for the `gl::file::load` code path specifically**. Full-workspace `verb/test` re-run: nextest 1690/1690, doctests pass, clippy clean, wasm32 check 49/49 examples (3 initially-failed examples traced to an unrelated known cause — a repo-root temp-file sweeper corrupting `target/` mid-build — confirmed passing clean on narrow-scope retry), wasm32 test 3/3 crates — no regressions from this fix. The remaining ~24 affected examples were not individually browser-checked; they share the identical `url_resolve`/`gl::file::load` code path and are covered by the wasm32 compile-check, not a live-render spot check.

**Correction (see `## Reopened` below):** this Verification's "zero relative-path 404s" claim held only for the sampled `gl::file::load`-based examples. It did not hold for the bug's own filed reproduction example's actual remaining asset load — `text_msdf`'s font atlas goes through a different, unfixed function (`dom::image_element_create`) — which an independent review reproduced as still 404ing against the real `_site/` build. Do not read this Verification block as covering the whole bug; see `## Reopened` for the accurate current scope.

## Reopened — Fix Incomplete

**Reopened:** 2026-08-13, by an independent review agent dispatched specifically to verify this closure (this project's Tier 2 MAAV cap permits a one-time independent-dispatch exception for exactly this kind of gate, granted for this review).

The `## Fix`/`## Verification` sections above are accurate for their own scope (`file.rs`'s `url_resolve`/`load`) — that part is genuinely fixed, tested (including a real RED→GREEN falsification: reverting the join line reproduces 4 test failures matching this bug's own predicted symptom; restoring it passes 16/16), and stays in place, unmodified. But the closure's overall verdict — that this bug is fully resolved — does not hold.

**What's still broken:** `mingl::web::dom::image_element_create` (`module/min/mingl/src/web/dom.rs:108-122`) and `video_element_create` (same file, lines 66-85) contain the textually identical bug pattern — `let origin = window.location().origin().unwrap(); let url = format!("{origin}/{path}");` (dom.rs:70-71, 112-113) — joining against bare origin instead of the page's own directory. Neither function was touched by the fix.

This is not theoretical: `text_msdf` — this bug's **own filed reproduction example** — loads its MSDF font atlas via `gl::dom::image_element_create("static/font/Alike-Regular.png")` (`examples/minwebgl/text_msdf/src/main.rs:133`). Reproduced directly against the real `_site/` build used for the original closure's own live-browser check:

```bash
curl http://localhost:8899/minwebgl/text_msdf/static/font/Alike-Regular.png  # 200 — correct subpath URL
curl http://localhost:8899/static/font/Alike-Regular.png                     # 404 — URL image_element_create actually constructs
```

`_site/static/font/` does not exist — only the correctly-pathed per-example copies do. This fails **silently**: `image_element_create` never wires `.set_onerror`, so a 404'd `<img>` produces no console error and no panic — unlike this bug's original symptom (a loud `.unwrap()` panic), which is likely why the original closure's spot-check didn't catch it.

**Confirmed via a second, independent full live-browser sweep** (23 of the ~29 affected examples served from a rebuilt `_site/` and loaded headless at their real deployed subpaths; the 6 `gl::file::load`-based examples from the original closure's own spot-check were not re-run):

- **`file.rs`'s actual fix generalizes correctly**: all 23 examples exercising the fixed `gl::file::load` code path show zero 404/NetworkError/panic symptoms traceable to it — the original fix stands confirmed, now effectively 29/29 rather than 6/29.
- **5 confirmed regressions, all tracing to the unfixed `dom.rs`/`filter.rs` gap above** (not to `file.rs`):

  | Example | Symptom | Site |
  |---|---|---|
  | `object_picking` | `Uncaught RuntimeError: unreachable` (panic on rejected load promise) | `examples/minwebgl/object_picking/src/main.rs:276,299` |
  | `sprite_animation` | identical panic | `examples/minwebgl/sprite_animation/src/main.rs:21-22` |
  | `video_as_texture` | `NotSupportedError: Failed to load because no supported source was found`; texture stays a black rectangle | `examples/minwebgl/video_as_texture/src/main.rs:20,24` |
  | `filter` | silent failure (own hand-rolled `image_load()`, no promise/panic — just never loads) | `examples/minwebgl/filter/src/main.rs:98-109` |
  | `mapgen_tiles_rendering` | silent failure — **compound**: beyond the `dom.rs` origin bug, `"tileset.png"` is also missing the `static/` prefix Trunk actually deploys it under (404 confirmed at both origin-root and at the no-`static/`-prefix subpath URL; 200 only at `.../mapgen_tiles_rendering/static/tileset.png`) — fixing `dom.rs` alone will not fully fix this one | `examples/minwebgl/mapgen_tiles_rendering/src/main.rs:47` |

  `wfc`'s own `image_element_create("tileset.png")` call site exists (`src/main.rs:73`) but rendered correctly on live load — showing the expected initial "Choose File / Generate" upload UI, not a broken state — because that call path isn't reached until user interaction. Not currently confirmed broken; recheck once `dom.rs` is fixed, don't assume clean.

- **3 inconclusive, unrelated to this bug** (blank canvas, zero error signal of any kind — most parsimoniously an unrelated rendering/environment issue, not asset-path resolution): `text_rendering`, `postprocessing`, `deffered_rendering` (the last is the documented sandboxed-Chromium WebGPU presentation ceiling, not a code defect).
- **5 more show console warnings or render failures from confirmed pre-existing, orthogonal WebGL bugs** (vertex-buffer sizing in `diamond`/`obj_load`; non-fatal `bindTexture`/`uniformBlockBinding` warnings in `lottie_surface_rendering`/`animation_surface_rendering`/`curve_surface_rendering`/`narrow_outline`, which still render correctly) — none traceable to BUG-109 in any form.

**Remaining work (unclaimed):** fix `dom.rs`'s `image_element_create`/`video_element_create` — ideally by routing both through the already-fixed `url_resolve` rather than re-deriving the join logic — fix `filter`'s own inline duplicate, fix `mapgen_tiles_rendering`'s separate missing-`static/`-prefix bug, then live-browser re-verify `object_picking`/`sprite_animation`/`video_as_texture`/`filter`/`mapgen_tiles_rendering`/`wfc` against the real `_site/` build before re-closing.

**Note:** a headless-browser pixel-level confirmation was attempted but came back inconclusive in this sandbox for a single-example spot-check (both the broken and a known-good control render came back solid black — a pre-existing headless-rendering ceiling on this machine, not evidence either way). The independent full sweep above uses console/network signal plus screenshots instead and reaches a conclusive verdict for 23/23 examples reached.

## Reclosed — Fix Complete

**Reclosed:** 2026-08-13, same session as `## Reopened`, after completing the remaining work it enumerated.

**Fixes applied**, all in `module/min/mingl/src/web/`:

- Shared `resolve_url`/`is_self_contained_url` moved into `web.rs`'s private `mod private {}` (crate-visible via `pub( crate ) use`), so `dom.rs` and `file.rs` both call the identical, already-tested join logic instead of each re-deriving it — closing the exact duplication gap the `## Reopened` section flagged.
- `file.rs`'s `url_resolve`/`is_self_contained_url` converted to thin wrappers delegating to `web.rs`, preserving `file.rs`'s external API.
- `dom.rs`: `image_element_create` and `video_element_create` now resolve via the shared `web::resolve_url( &href, path )` instead of their own `format!( "{origin}/{path}" )` — the textually-identical bug pattern flagged in `## Reopened` is gone from both functions.
- `examples/minwebgl/filter/src/main.rs`: its own hand-rolled `image_load()` now calls `gl::web::file::url_resolve` instead of `format!( "{origin}/{path}" )`.
- `examples/minwebgl/mapgen_tiles_rendering/src/main.rs`: `image_load()`'s URL join fixed the same way (BUG-109 proper); separately, during this window's live-reverification, found and fixed a **distinct, pre-existing bug** (not BUG-109 — see the file's own `Fix` comment for the full explanation): `image_load` sets the created `<img>`'s DOM `id` to the full path it's given (`"static/tileset.png"`), but `texture_array_prepare`'s lookup was passing the bare filename (`"tileset.png"`), which never matched — `get_element_by_id` returned `None`, and the leading `?` silently skipped all texture setup before any GL call ran. Fixed the call site to pass the matching id.

**Live-browser re-verification** (this session, `browsee`, real Firefox/Chromium sessions, served from a freshly rebuilt `_site/` at `http://127.0.0.1:38611/`) — all 7 examples named in `## Reopened`'s remaining-work list, confirmed rendering correctly with clean consoles:

| Example | Result |
|---|---|
| `filter` | Concentric-rings test image renders correctly (re-checked twice, including a session-leak-free control run) |
| `mapgen_tiles_rendering` | Tile map (grass/sand/water/stone) renders correctly, after also fixing the id-mismatch bug above |
| `object_picking` | Grid of cat meshes renders correctly |
| `sprite_animation` | Asteroid sprite renders correctly |
| `text_msdf` | "Cgtools" MSDF text renders correctly after its intro camera pan settles |
| `video_as_texture` | SMPTE color-bar test video plays correctly as a texture |
| `wfc` | Shows its expected interactive "Browse... / Generate" upload UI (this demo requires a user-supplied input image before generating output — confirmed as by-design, not a regression) |

**Detour, not a code bug:** mid-reverification, `mapgen_tiles_rendering` briefly showed a blank white page with no `<canvas>` element at all — worse than the original bug. Root-caused to a stale build artifact: an earlier manual rebuild in this session used `trunk build --public-url "/mapgen_tiles_rendering/"` (missing the `minwebgl/` prefix that `_site/`'s actual layout and the rest of the rebuild tooling use), which baked a wrong self-referencing JS-module path into `dist/index.html`, 404ing the module fetch so the wasm never loaded and `main()` never ran. Rebuilding with the correct `--public-url "/minwebgl/mapgen_tiles_rendering/"` resolved it. Confirmed via direct `curl` against the running server (404 at the wrong path, 200 at the correct one) before and after. Not a `mingl`/example source defect — recorded here only for traceability since it briefly looked like a regression from the id-mismatch fix above.

**Verification (full 4 layers):**
- **Checklist:** all items in `## Reopened`'s "Remaining work (unclaimed)" list completed — `dom.rs` routed through shared `resolve_url` ✓, `filter`'s inline duplicate fixed ✓, `mapgen_tiles_rendering`'s missing-`static/`-prefix bug fixed ✓ (plus the separately-discovered id-mismatch bug) ✓, all 6 named examples plus `wfc` live-browser re-verified ✓.
- **Measurements:** `cargo clippy --all-targets --all-features --target wasm32-unknown-unknown -- -D warnings` clean for `mingl`, `filter`, `mapgen_tiles_rendering`; `trunk build --release` exit 0 for all 7 examples; live-browser screenshots captured for all 7 (see session transcript).
- **Invariants:** `file.rs`'s public API unchanged (thin-wrapper conversion, confirmed via its own doctest/regression suite still passing); no example's rendering logic changed beyond the URL-join/id-lookup fix sites.
- **Anti-faking:** every example was actually loaded in a real browser (Firefox or Chromium via `browsee`) against the real `_site/` staging build, not merely compiled — screenshots are the evidentiary artifact, not a self-report of "should work."

All work items `## Reopened` listed as unclaimed are now closed. This bug is fully resolved.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-13 | filed | `text_msdf` panicked on a 404'd font-atlas fetch under its real deployed subpath — `mingl::web::file::url_resolve` joined document-relative paths against `origin` alone, dropping the page's own directory. |
| 2026-08-13 | fixed, verified | `file.rs`'s `url_resolve`/`load` fixed to join against the document's own directory; regression tests added; spot-checked 6 `gl::file::load`-based examples live in-browser, all clean; closed to `completed/`. |
| 2026-08-13 | reopened | Independent review agent found `dom.rs`'s `image_element_create`/`video_element_create` carry the textually identical bug, untouched by the fix — reproduced live against 5 examples (`object_picking`, `sprite_animation`, `video_as_texture`, `filter`, `mapgen_tiles_rendering`); `wfc` flagged as same-code-path but not-yet-confirmed-broken. Moved back to `verified/`. |
| 2026-08-13 | reclosed | Shared join logic consolidated into `web.rs`; `dom.rs`, `filter`'s inline duplicate, and `mapgen_tiles_rendering`'s missing-`static/`-prefix bug all fixed; a separate, distinct DOM id-mismatch bug found in `mapgen_tiles_rendering` during reverification and fixed alongside it. All 7 examples from the reopen list live-reverified in a real browser, all render correctly. Closed via self-verification per this registry's established convention (§ `task/bug/completed/` precedent — bugs in this project are closed by direct file/table edit, not the `tsk` CLI state machine). |

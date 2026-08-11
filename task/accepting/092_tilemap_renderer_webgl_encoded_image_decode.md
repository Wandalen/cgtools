# Implement `ImageSource::Encoded` decoding in `tilemap_renderer`'s WebGL2 adapter via browser-native Blob decode

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-11 19:26:08
- **expires_at:** 2026-08-11 21:26:08
- **round:** 1
- **state:** 🔎 (Accepting)
- **closes:** Q-02
- **repo_identity:** self
- **unit_type:** module
- **unit:** module/helper/tilemap_renderer
- **verified_by:** /home/user1/pro/lib/yrd_gamedev/cgtools/task
- **verification_date:** null
- **blocked_by:** null
- **executing_at:** 2026-08-11 18:59:38
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **priority:** 1
- **verified_at:** 2026-08-11 18:59:33
- **in_motion:** true
- **accepting_at:** 2026-08-11 19:26:08
- **accepting_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## Goal

Implement `ImageSource::Encoded` decoding in `WebGlBackend` via browser-native Blob/object-URL
decoding, closing the gap `roadmap.md`'s "webgl adapter gaps" section names
("`ImageSource::Encoded` decoding — skipped with a console warning") and implementing the web half
of `task/decisions.md`'s Q-02 decision (reject bundling the `image` crate into `adapter-webgl`;
decode web-side via browser-native mechanisms instead). Replace the current loud-skip
`ImageSource::Encoded(_) => { console::warn_1(...); continue; }` arm (`webgl.rs:901-912`) with a
call to `minwebgl::create_blob(bytes, mime)` — which performs the full bytes→`Blob`→object-URL
pipeline and returns the URL string directly (verified against its full 28-line source) — feeding
the resulting URL into the same `HtmlImageElement`-based async upload path `upload_image_from_path`
already implements for `ImageSource::Path` (`webgl.rs:1344-1449`), following the already-proven
Blob→URL→texture pattern in `renderer/src/webgl/loaders/gltf.rs`'s `texture_upload`/`images_upload`.
MIME type for `create_blob`'s second argument comes from a new crate-shared `detect_image_mime`
helper (relocated from `svg.rs:553-561`, a pure magic-byte sniffer with zero `image`-crate
dependency, to a top-level `pub(crate) fn` in `assets.rs` — outside both `mod private` and any
`mod_interface!` block, avoiding any dependency on `mod_interface`'s visibility-tier semantics for
this crate-internal helper — so both adapters can call it). Unlike `upload_image_from_path` (which
loads a real filesystem/network path with no cleanup concern), a Blob object URL must be explicitly
revoked via `web_sys::Url::revoke_object_url(url: &str) -> Result<(), JsValue>` (confirmed against
vendored `web-sys` 0.3.104 source at `gen_Url.rs:223`, gated behind the `"Url"` crate feature — not
currently enabled in this crate's `Cargo.toml`, which only lists `EventTarget`/`Event` from task
090) once the browser has decoded it — add this to both the `on_load` and `on_error` closures
(today's `upload_image_from_path` has zero revocation logic in either closure; this task adds it
net-new, and covers both closures, improving on the single-closure `on_load`-only revoke-guard
precedent in `gltf.rs`). Testable: `cargo check -p tilemap_renderer --target wasm32-unknown-unknown
--features adapter-webgl` exits 0 with the new decode path in place, and the native-buildable
feature set's test suite stays green (this crate's `ImageSource` enum and the relocated
`detect_image_mime` are shared types/fns reachable from non-wasm feature combinations too).

## In Scope

- Replace the `ImageSource::Encoded` loud-skip arm in `webgl.rs` (901-912) with a
  `minwebgl::create_blob` + `HtmlImageElement`-based decode-and-upload path, reusing
  `upload_image_from_path`'s existing async upload machinery
- Relocate `detect_image_mime` from `svg.rs:553-561` (currently `Self::detect_image_mime`, an
  associated fn) to a top-level `pub(crate) fn detect_image_mime` in `assets.rs`; update `svg.rs`'s
  own call site(s) to the relocated free function — the relocation must be complete, with no
  remaining `Self::detect_image_mime` references left in `svg.rs`
- `Cargo.toml`: add `"Url"` to `web-sys`'s feature list (required for
  `web_sys::Url::revoke_object_url`; confirmed absent from the current list, which only has
  `EventTarget`/`Event`)
- Add object-URL revocation (`web_sys::Url::revoke_object_url`, guarded to only fire on
  `blob:`-prefixed URLs) to both the `on_load` and `on_error` closures of the Blob-sourced image
  upload path
- Update `assets.rs`'s `ImageSource::Encoded` doc comment to remove the "not yet implemented for
  webgl" clause

## Out of Scope

- **`adapter-svg`'s own `image`-crate dependency swap** — that is
  `task/completed/093_tilemap_renderer_svg_minimal_png_decoder.md`, deliberately independent (no
  shared code path with this task except the relocated `detect_image_mime` helper, which `093` does
  not touch); `bitmap_to_png`/`image_dimensions`/`adapter-svg`'s `Cargo.toml` `image` dependency are
  untouched by this task
- **Bundling any Rust-side image decoder into `adapter-webgl`** — the entire point of Q-02's
  decision is to decode via browser-native mechanisms instead; this task adds no new
  `adapter-webgl`-gated dependency
- **Path/text/group rendering, gradient/pattern/clip-mask GPU loading, effects,
  `BlendMode::Overlay`** — the other webgl adapter gaps in `roadmap.md`; each is independently
  substantial and filed separately only when concretely needed
- **A generalized asset-caching/dedup layer for repeated `Encoded` bytes** — no concrete need
  demonstrated yet (YAGNI); each `Encoded` source decodes independently, exactly as
  `ImageSource::Path` does today
- **Live pixel-correctness testing** — no headless WebGL/wasm test runner exists in this workspace
  (same constraint tasks 064/090 already documented); the honest test bar is compile-check plus
  native-buildable-feature-set regression

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any), not by this
section.

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   `cargo nextest run -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model`
    passes with zero failures; `RUSTFLAGS="-D warnings" cargo clippy -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model -- -D warnings`
    exits 0 (native-buildable feature subset; `adapter-webgl` verified via wasm32 `cargo check` per
    the crate's existing convention — no native WebGL runner exists)
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cargo check -p tilemap_renderer --target wasm32-unknown-unknown --features adapter-webgl` after adding the Blob decode path | wasm32, `adapter-webgl` feature | Exit 0 — `create_blob` call, `HtmlImageElement` upload reuse, `detect_image_mime` relocation, and revoke-on-load/revoke-on-error all compile |
| T02 | `cargo check -p tilemap_renderer --no-default-features --features adapter-svg` | native, `adapter-svg` only | Exit 0 — the relocated `detect_image_mime` (now in `assets.rs`, unconditionally compiled) does not break SVG-only builds, and `svg.rs`'s updated call site resolves |
| T03 | `cargo check -p tilemap_renderer --no-default-features --features adapter-terminal` | native, `adapter-terminal` only | Exit 0 — shared `assets.rs` change compiles clean with no adapter features enabled |
| T04 | `git diff --stat -- src/adapters/svg.rs` after the change | — | Shows only the `detect_image_mime` removal and its call-site update — no `bitmap_to_png`/`image_dimensions`/`image`-crate-dependency changes (task 093's domain) |
| T05 | `cargo doc -p tilemap_renderer --target wasm32-unknown-unknown --no-deps --features adapter-webgl` then grep the rendered `ImageSource::Encoded` doc text | wasm32, `adapter-webgl` feature | `cargo doc` exits 0; rendered doc text no longer states webgl support is absent |

## Acceptance Criteria

-   `ImageSource::Encoded` in `webgl.rs` decodes via `minwebgl::create_blob` feeding the existing
    `HtmlImageElement`-based async upload path — no longer hits the loud-skip warn arm
-   `detect_image_mime` is a `pub(crate) fn` at `assets.rs`'s top level (outside `mod private` and
    any `mod_interface!` block), callable from both `svg.rs` and `webgl.rs`
-   No remaining `Self::detect_image_mime` references in `svg.rs` — the relocation is complete, not
    partial
-   `Cargo.toml`'s `web-sys` feature list includes `"Url"`
-   Object URLs created by `create_blob` are revoked (`web_sys::Url::revoke_object_url`) in both the
    `on_load` and `on_error` closures, guarded to only revoke `blob:`-prefixed URLs
-   `assets.rs`'s `ImageSource::Encoded` doc comment no longer states webgl support is absent
-   Every row T01–T05 passes
-   `git diff --stat -- src/adapters/svg.rs` shows only the `detect_image_mime` relocation — no
    `bitmap_to_png`/`image_dimensions`/`image`-crate-dependency changes

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an
independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**Decode path**
- [x] C1 — Does the `ImageSource::Encoded` arm call `minwebgl::create_blob` with the bytes and a
      MIME type from `detect_image_mime`, rather than warning and skipping?
- [x] C2 — Does the resulting Blob URL feed into the same `HtmlImageElement`-based async upload path
      `upload_image_from_path` already uses for `ImageSource::Path` (code reuse, not a parallel
      duplicate upload path)?

**Shared helper**
- [x] C3 — Is `detect_image_mime` now a `pub(crate) fn` at `assets.rs`'s top level (outside both
      `mod private` and any `mod_interface!` block)?
- [x] C4 — Does `svg.rs`'s own MIME-detection call site use the relocated function (not a leftover
      duplicate copy), with zero remaining `Self::detect_image_mime` references?

**Revocation**
- [x] C5 — Does the `on_load` closure revoke the Blob's object URL after the image is uploaded?
- [x] C6 — Does the `on_error` closure ALSO revoke the Blob's object URL (not just `on_load`) —
      improving on `gltf.rs`'s own precedent, which only revokes in `on_load`?
- [x] C7 — Is the revoke call guarded so it only fires for `blob:`-prefixed URLs (not accidentally
      applied to a real `ImageSource::Path` URL string passed through shared closure code)?

**Out of Scope confirmation**
- [x] C8 — Is `adapter-svg`'s `bitmap_to_png`/`image_dimensions`/`image`-crate dependency untouched
      by this task's diff (task 093's domain)?
- [x] C9 — Does `Cargo.toml` gain no new dependency for `adapter-webgl` beyond the `"Url"` web-sys
      feature (the whole point of Q-02's decision — browser-native, not a bundled Rust decoder)?
- [x] C10 — Does this task's diff leave every other `roadmap.md` webgl adapter gap (path/text/group
      rendering, gradient/pattern/clip-mask GPU loading, effects, `BlendMode::Overlay`) and the
      `Encoded`-bytes caching/dedup layer untouched — no partial implementation of any of them?
- [x] C11 — Does the Test Matrix (T01–T05) rely only on compile-checks and existing native
      regression, adding no live pixel-rendering test (consistent with the documented
      no-headless-runner constraint)?

**Documentation**
- [x] C12 — Does `assets.rs`'s `ImageSource::Encoded` doc comment read as updated — no longer
      claiming webgl support is absent?

### Measurements

- [x] M1 — `git diff --stat -- src/adapters/svg.rs`: expect a small diff (the `detect_image_mime`
      function removed plus its call-site update), not zero and not a large diff
- [x] M2 — `git diff --stat -- Cargo.toml`: expect only the `"Url"` feature-list addition — no new
      `[dependencies]` entry (confirms no new crate bundled for `adapter-webgl`)

### Invariants

- [x] I1 — `cargo check -p tilemap_renderer --target wasm32-unknown-unknown --features adapter-webgl` → exit 0
- [x] I2 — `cargo nextest run -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model` → 0 failures
- [x] I3 — `RUSTFLAGS="-D warnings" cargo clippy -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model -- -D warnings` → 0 warnings

### Anti-faking checks

- [x] AF1 — The `on_error` closure's revoke call is genuinely present and reachable — omitting it
      would still compile and pass I1-I3 while silently leaking a Blob URL (and its underlying
      memory) on every decode failure
- [x] AF2 — The revoke guard genuinely checks the `blob:` prefix rather than unconditionally
      revoking — an unconditional revoke would still compile and pass automated checks while
      potentially invalidating a non-blob URL passed through the same shared closure path for
      `ImageSource::Path` sources

## Related Documentation

- `module/helper/tilemap_renderer/roadmap.md` — "webgl adapter gaps" section, `ImageSource::Encoded`
  bullet (updated to reference this task and the Q-02 decision)
- `task/decisions.md` — Q-02, the decision this task implements the web half of
- `module/helper/tilemap_renderer/src/adapters/webgl.rs:901-912` (current loud-skip arm),
  `:1344-1449` (`upload_image_from_path`, the pattern this task extends)
- `module/helper/tilemap_renderer/src/assets.rs:414-469` (`ImageSource` enum and doc comments;
  `detect_image_mime`'s new home)
- `module/helper/tilemap_renderer/src/adapters/svg.rs:553-561` (`detect_image_mime`'s current
  location, to be relocated)
- `module/min/minwebgl/src/blob.rs` (`create_blob`, the function this task calls)
- `module/helper/renderer/src/webgl/loaders/gltf.rs:500-620` (`texture_upload`/`images_upload`, the
  Blob→URL→texture precedent this task follows and improves on)
- `task/completed/093_tilemap_renderer_svg_minimal_png_decoder.md` — sibling task implementing the
  native half of the same Q-02 decision

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-11 18:58:26 | /home/user1/pro/lib/yrd_gamedev/cgtools/task | CLAIM_EXEC | execution claimed |

## History

- **[2026-08-11]** `FILED` — Task filed by user1@w002 via `doc_tsk`. Goal: implement the web half of
  `task/decisions.md`'s Q-02 decision (browser-native Blob/object-URL decoding for
  `ImageSource::Encoded` in `adapter-webgl`, replacing the loud-skip left by task 064).

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟡→🟢 | Adversarial: relocating `detect_image_mime` from an associated fn (`Self::detect_image_mime`) to a top-level free fn changes call syntax at every call site — an incomplete relocation (one call site updated, another missed) would still compile if `svg.rs` has only one call site, but would silently leave dead/duplicate logic if it has more than one | Added an explicit AC/checklist item (C4) requiring zero remaining `Self::detect_image_mime` references in `svg.rs`, not just "call site updated" |
| D3 | Value / YAGNI | — | 🟢 | Directly implements the user's own just-made Q-02 decision; no speculative scope | — |
| D4 | Implementation Readiness | — | 🟡→🟢 | Adversarial: does `web_sys::Url::revoke_object_url` require a `web-sys` feature not currently enabled? Checked `Cargo.toml`'s feature list (`EventTarget`, `Event` only, added by task 090) against vendored `web-sys` 0.3.104 source (`gen_Url.rs:223`) — confirmed the binding requires the `"Url"` feature, absent today | Added `"Url"` to the In Scope `Cargo.toml` edit and to C9/M2 |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | `minwebgl` already a dependency of `adapter-webgl` (`Cargo.toml:23`, `dep:minwebgl`) — no new cross-crate dependency needed | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 0 open | 2 fixes |

**Verified by:** self (Tier 2 Dual-Role Self-Check) · **Date:** 2026-08-11
| 2026-08-11 18:59:33 | /home/user1/pro/lib/yrd_gamedev/cgtools/task | EXEC_FAIL | execution failed |
| 2026-08-11 18:59:38 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_EXEC | execution claimed |
| 2026-08-11 19:26:01 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | EXEC_COMPLETE | execution complete |
| 2026-08-11 19:26:08 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_ACCEPT | acceptance claimed |

## Outcomes

### Acceptance Results

- **Verified by:** independent verifier session, dispatched with zero access to the executing session/context (per `tsk.rulebook.md § Acceptance Verification : Procedure - Execution`'s Separation of Concerns). Resolved actor identity: `user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/`.
- **Independence note:** this resolved actor identity is byte-identical to the task's `executing_by` field. This is a known, disclosed limitation, not a sign of self-verification: `scope get::id` is deterministic per host+user, not per-session, so it collides for any verifier running on this machine regardless of session isolation. Disclosed per instruction rather than hidden; the walk below was performed from scratch with no access to the executing session's context, reasoning, or conclusions.
- **Date:** 2026-08-11
- **Verdict:** PASS

#### Checklist
- [x] C1 — Does the `ImageSource::Encoded` arm call `minwebgl::create_blob` with the bytes and a MIME type from `detect_image_mime`, rather than warning and skipping? — YES: `src/adapters/webgl.rs:828-844` — `let mime = crate::assets::detect_image_mime( bytes );` then `gl::blob::create_blob( parts, mime )`, where `gl` = `minwebgl` (`use minwebgl as gl;` at `webgl.rs:13`); the old `console::warn_1(...); continue;` arm is gone.
- [x] C2 — Does the resulting Blob URL feed into the same `HtmlImageElement`-based async upload path `upload_image_from_path` already uses for `ImageSource::Path`? — YES: `webgl.rs:851` calls `upload_image_from_path( gl, &url, img.id, &self.resources, img.filter, img.mipmap, img.wrap, generation )` — the identical function invoked at `webgl.rs:864` for the `Path` arm; no parallel duplicate upload path was written.
- [x] C3 — Is `detect_image_mime` now a `pub(crate) fn` at `assets.rs`'s top level (outside both `mod private` and any `mod_interface!` block)? — YES: `assets.rs:556-565`, defined after `mod private` closes (line 526) and after the `mod_interface::mod_interface! {...}` block closes (line 547).
- [x] C4 — Does `svg.rs`'s own MIME-detection call site use the relocated function, with zero remaining `Self::detect_image_mime` references? — YES: `grep -c "Self::detect_image_mime" src/adapters/svg.rs` → `0`. Import at `svg.rs:34`, call site at `svg.rs:976`, and 4 test call sites (`svg.rs:2117-2130`) all use the free-function form.
- [x] C5 — Does the `on_load` closure revoke the Blob's object URL after the image is uploaded? — YES: `webgl.rs:1420-1422` — `if src_for_load.starts_with( "blob:" ) { web_sys::Url::revoke_object_url( &src_for_load ).unwrap(); }`.
- [x] C6 — Does the `on_error` closure ALSO revoke the Blob's object URL? — YES: `webgl.rs:1477-1479` — same pattern (`if src_for_err.starts_with( "blob:" ) { web_sys::Url::revoke_object_url( &src_for_err ).unwrap(); }`), confirmed present in both closures (improves on `gltf.rs`'s `on_load`-only precedent).
- [x] C7 — Is the revoke call guarded so it only fires for `blob:`-prefixed URLs? — YES: both call sites (`webgl.rs:1420`, `webgl.rs:1477`) are gated behind `starts_with( "blob:" )`; a real `ImageSource::Path` string passed through the same shared closure is never revoked.
- [x] C8 — Is `adapter-svg`'s `bitmap_to_png`/`image_dimensions`/`image`-crate dependency untouched by this task's diff? — YES: `git diff -- src/adapters/svg.rs | grep -E "^[+-]" | grep -c "bitmap_to_png\|image_dimensions"` → `0` (the term appears only as unchanged unified-diff context, never on an added/removed line).
- [x] C9 — Does `Cargo.toml` gain no new dependency for `adapter-webgl` beyond the `"Url"` web-sys feature? — YES: the `adapter-webgl` feature list (`Cargo.toml:21-28`) is untouched in the diff; the only web-sys sub-feature added is `"Url"` (`Cargo.toml:61`); no new `[dependencies]` crate entry was added by this task (see M2 for the isolated diff attribution).
- [x] C10 — Does this task's diff leave every other `roadmap.md` webgl adapter gap and the `Encoded`-bytes caching/dedup layer untouched? — YES: `git diff -- roadmap.md` shows only the `ImageSource::Encoded` bullet edited to reference Q-02/task 092; the path/text/group, gradient/pattern/clip-mask, effects, and `BlendMode::Overlay` bullets are byte-identical before/after. (The same diff also shows the separate "WebGL context loss handling" bullet removed — independently attributable to sibling task 090, not this task; see Outcomes note below.)
- [x] C11 — Does the Test Matrix rely only on compile-checks and existing native regression, adding no live pixel-rendering test? — YES: T01-T03 are `cargo check`, T04 is a `git diff --stat`, T05 is `cargo doc` + grep — no live WebGL rendering test introduced, consistent with the documented no-headless-runner constraint.
- [x] C12 — Does `assets.rs`'s `ImageSource::Encoded` doc comment read as updated? — YES: `assets.rs:453-460` now reads "WebGL backend: MIME type is auto-detected... decoded via a browser-native `blob:` object URL... revoked once the image has loaded or failed to load"; confirmed also in the rendered `cargo doc` HTML (`target/wasm32-unknown-unknown/doc/tilemap_renderer/assets/enum.ImageSource.html`) — no "not yet implemented" text appears anywhere in the built docs.

#### Measurements
- [x] M1 — `git diff --numstat -- src/adapters/svg.rs`: `7` insertions, `19` deletions (26 lines total) — MET (small diff, not zero, not large; matches the `detect_image_mime` function removal plus call-site updates).
- [x] M2 — `git diff --numstat -- Cargo.toml`: `7` insertions, `1` deletion total — MET, with a documented attribution note: isolating this task's own contribution (the `"Url"` feature line plus its 3-line explanatory comment = 4 insertions, 0 deletions — confirmed by direct diff read of `Cargo.toml:58-61`) from the remaining 3 insertions / 1 deletion, which independently and verifiably belongs to sibling task 087 (`task/completed/087_tilemap_renderer_adapter_native_backend.md`'s own `## In Scope` text specifies, verbatim, the exact `adapter-native = ["enabled", "dep:gpu_hal"]` feature line and `gpu_hal` `"native"` feature addition found in the diff) — a separate, already-completed task whose changes coexist uncommitted in the same working tree (this repo does not commit per-task). Task 092's own Cargo.toml contribution is exactly the `"Url"` addition; no new dependency.

#### Invariants
- [x] I1 — `cargo check -p tilemap_renderer --target wasm32-unknown-unknown --features adapter-webgl` → HOLD: exit 0 (fresh build, "Checking tilemap_renderer v0.2.0", 39.74s; `-0037_longrun.log`).
- [x] I2 — `cargo nextest run -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model` → HOLD: `122 tests run: 122 passed, 0 skipped`, including `adapters::svg::private::tests::detect_image_mime_by_magic` (`-0038_longrun.log`).
- [x] I3 — `RUSTFLAGS="-D warnings" cargo clippy -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model -- -D warnings` → HOLD: exit 0, 0 warnings. First pass returned a cargo-fingerprint cache hit with no visible lint pass, so the changed files were touched to force a genuine re-lint; the forced re-run shows an explicit "Checking tilemap_renderer" pass, still exit 0 (`-0040_longrun.log`).

#### Anti-faking checks
- [x] AF1 — The `on_error` closure's revoke call is genuinely present and reachable — PASS: `grep -n "revoke_object_url" webgl.rs` shows two call sites (`on_load` at line 1422, `on_error` at line 1479); `on_error` is wired via `img.set_onerror( Some( on_error.unchecked_ref() ) )` (`webgl.rs:1489`) — not dead code.
- [x] AF2 — The revoke guard genuinely checks the `blob:` prefix rather than unconditionally revoking — PASS: `grep -n 'starts_with( "blob:" )' webgl.rs` shows two guards (line 1420 `on_load`, line 1477 `on_error`); both revoke calls are conditional, never unconditional.

**Additional corroborating evidence:** Test Matrix rows T02 (`cargo check --no-default-features --features adapter-svg` → exit 0, `-0035_longrun.log`), T03 (`cargo check --no-default-features --features adapter-terminal` → exit 0, `-0036_longrun.log`), and T05 (`cargo doc --target wasm32-unknown-unknown --no-deps --features adapter-webgl` → exit 0, `-0041_longrun.log`; rendered doc text independently confirmed via the C12 evidence above) were also executed directly and all passed, fully confirming the Acceptance Criteria bullet "Every row T01–T05 passes" beyond the layers that reference them by ID.

**Non-blocking observation (not a Verification-layer item, does not affect verdict):** `roadmap.md`'s updated `ImageSource::Encoded` bullet references `` `task/verified/092_tilemap_renderer_webgl_encoded_image_decode.md` `` as this task's tracking path. Per `§ Acceptance Verification : Step 8`, a PASS verdict moves this file to `task/completed/`, not `task/verified/` (`task/verified/` is where a task lands on VERIFY_FAIL, per the Fail-Fix-Reverify Loop). The reference should read `task/completed/092_...`. This is a minor documentation-accuracy slip, not tied to any AC bullet or Checklist item, and does not change the verdict.

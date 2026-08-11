# Handle WebGL context loss in `tilemap_renderer`'s WebGL2 adapter

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** module/helper/tilemap_renderer
- **verified_by:** independent verifier (general-purpose Agent, blind dispatch)
- **verification_date:** 2026-08-11
- **blocked_by:** null
- **executing_at:** 2026-08-11 15:41:40
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/

## Goal

Implement WebGL context-loss detection and safe degradation in `WebGlBackend`, closing the gap
`roadmap.md`'s "webgl adapter gaps" section names: "WebGL context loss handling
(`webglcontextlost` / `webglcontextrestored` events)". Register persistent `webglcontextlost` /
`webglcontextrestored` listeners on the canvas reachable from `WebGlBackend`'s existing `gl::GL`
context at construction time (`webgl.rs:282`), following the same `Closure`-based event-registration
idiom the adapter already uses for async image loading (`webgl.rs:1290-1360`,
`Closure::once_into_js`) — though a *persistent* form is needed here (`Closure::wrap` + `.forget()`
or equivalent, confirmed against `wasm-bindgen`'s own API at implementation time) since context can
be lost and restored more than once in a session, unlike a one-shot image load. On loss: call the
event's `prevent_default()` (the WebGL spec's own opt-in signal for the browser to attempt
restoration at all — omitting it means the browser never tries) and set an internal flag so
`submit`/`output` return a new `RenderError::ContextLost` instead of driving calls into an invalid
context. On restoration: clear the flag; callers re-establish GPU state via the `Backend` trait's
existing `load_assets` contract (already documented as "safe to call multiple times... backends
must clear and reload all GPU/SVG state" — restoration requires exactly this same reload, no new
trait method needed). Testable: `cargo check -p tilemap_renderer --target wasm32-unknown-unknown
--features adapter-webgl` exits 0 with the new listeners, flag, and `RenderError` variant present,
and the native-buildable feature set's test suite stays green with the new enum variant in place.

## In Scope

- Persistent `webglcontextlost` / `webglcontextrestored` event listener registration on the canvas
  reachable from `WebGlBackend`'s `gl::GL` context, added in `WebGlBackend::new` (`webgl.rs:282`)
- `webglcontextlost` handler: calls the event's `prevent_default()`; sets an internal lost-state
  flag on the backend (e.g. a `Cell<bool>` or `Rc<Cell<bool>>` field, consistent with the crate's
  existing `Rc<RefCell<GpuResources>>` interior-mutability pattern for state shared with closures)
- New `RenderError::ContextLost` variant in `backend.rs`'s `RenderError` enum (20-28), with a doc
  comment stating the caller's required re-load-via-`load_assets` contract on restoration
- `submit` and `output` (`backend.rs:169`, `177`) check the lost-state flag *before* issuing any GL
  call in their `WebGlBackend` implementations, returning `RenderError::ContextLost` instead
- `webglcontextrestored` handler: clears the lost-state flag

## Out of Scope

- **Automatic asset re-upload on restoration** — the `Backend` trait's existing `load_assets`
  contract already covers this ("safe to call multiple times... clear and reload all GPU/SVG
  state"); the caller decides when to re-call it, exactly as today. This task only makes loss
  observable and safe, not self-healing
- **A new `Backend` trait method for proactively querying lost-state** (e.g. `is_context_lost()`) —
  `submit`/`output` returning `RenderError::ContextLost` is sufficient signal; a poll-style query
  method is speculative until a caller concretely needs one
- **SVG and Terminal adapters** — context loss is a WebGL-only browser/GPU concept; both other
  backends are unaffected and untouched by this task
- **`ImageSource::Encoded` decoding, path/text/group rendering, gradient/pattern/clip-mask GPU
  loading, effects, `BlendMode::Overlay`** — the other gaps in roadmap.md's "webgl adapter gaps"
  section; each is independently substantial and will be filed as its own task only when concretely
  needed. `ImageSource::Encoded` specifically is blocked on `task/decisions.md`'s Q-02 (unowned
  wasm-binary-size decision), not filed here
- **Forced context-loss testing tools** (e.g. driving the `WEBGL_lose_context` extension end-to-end)
  — no headless WebGL runner exists in this workspace (same constraint task 064 already documented);
  the honest test bar here is compile-check plus native-buildable-feature-set regression, not a live
  loss/restore integration test

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
| T01 | `cargo check -p tilemap_renderer --target wasm32-unknown-unknown --features adapter-webgl` after adding listeners/flag/variant | wasm32, `adapter-webgl` feature | Exit 0 — event registration, guard checks in `submit`/`output`, and the new `RenderError` variant all compile |
| T02 | `cargo check -p tilemap_renderer --no-default-features --features adapter-svg` | native, `adapter-svg` only | Exit 0 — the new `RenderError::ContextLost` variant (shared `backend.rs` type) does not break SVG-only builds |
| T03 | `cargo check -p tilemap_renderer --no-default-features --features adapter-terminal` | native, `adapter-terminal` only | Exit 0 — same non-exhaustive-match confirmation for the terminal stub |
| T04 | `git diff --stat -- src/adapters/svg.rs src/adapters/terminal.rs` after the change | — | Empty — context loss is WebGL-only, both other adapters untouched |
| T05 | `cargo doc -p tilemap_renderer --target wasm32-unknown-unknown --no-deps --features adapter-webgl` then `grep -A3 "ContextLost" target/wasm32-unknown-unknown/doc/tilemap_renderer/enum.RenderError.html` | wasm32, `adapter-webgl` feature | `cargo doc` exits 0 (no broken intra-doc links); rendered doc text for the variant mentions re-calling `load_assets`, matching that method's own existing "safe to call multiple times... clear and reload" contract |

## Acceptance Criteria

-   `WebGlBackend::new` registers persistent `webglcontextlost` and `webglcontextrestored` listeners
    on the canvas reachable from its `gl::GL` context
-   `webglcontextlost` handler calls `prevent_default()` on the event and sets the lost-state flag
-   `submit` and `output` return `RenderError::ContextLost` when the flag is set, without issuing any
    GL call first
-   `webglcontextrestored` handler clears the flag
-   `backend.rs`'s `RenderError::ContextLost` doc comment states the re-load-via-`load_assets`
    contract
-   Every row T01–T05 passes
-   `git diff --stat -- src/adapters/svg.rs src/adapters/terminal.rs` is empty

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an
independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**Event registration**
- [x] C1 — Does `WebGlBackend::new` register both `webglcontextlost` AND `webglcontextrestored`
      listeners (not just one)?
- [x] C2 — Does the `webglcontextlost` handler call `prevent_default()` on the event? (without it,
      the browser never attempts restoration at all, per the WebGL spec)
- [x] C5 — Is the closure registration a persistent form (`Closure::wrap` + `.forget()` or
      equivalent), not `Closure::once_into_js` (which fires only once — wrong for a recurring event)?

**Guard behavior**
- [x] C3 — Do `submit` and `output` check the lost-state flag *before* issuing any GL call, returning
      `RenderError::ContextLost` instead when set?
- [x] C4 — Does `webglcontextrestored` clear the flag, so a caller's next `submit`/`output` (after
      re-calling `load_assets`) proceeds normally?

**Out of Scope confirmation**
- [x] C6 — Are `svg.rs` and `terminal.rs` untouched by this task's diff?
- [x] C7 — Was no new `Backend` trait method added (confirms the "reuse `submit`/`output` error
      return, no proactive query method" boundary held)?
- [x] C8 — Does the `webglcontextrestored` handler only clear the lost-state flag — no automatic
      `load_assets` call added on the caller's behalf (confirms restoration stays caller-driven, not
      self-healing)?
- [x] C9 — Is `git diff --stat` limited to `webgl.rs`/`backend.rs` only — no touched code path for
      `ImageSource::Encoded`, path/text/group rendering, gradient/pattern/clip-mask loading, effects,
      or `BlendMode::Overlay` (the other roadmap.md gaps this task does not address)?

### Measurements

- [x] M1 — `RenderError` enum change size: `git diff --stat -- src/backend.rs`
      (before: 0 lines changed — enum has 3 variants prior to this task)
- [x] M2 — `git diff --stat -- src/adapters/svg.rs src/adapters/terminal.rs`: expected empty

### Invariants

- [x] I1 — `cargo check -p tilemap_renderer --target wasm32-unknown-unknown --features adapter-webgl` → exit 0
- [x] I2 — `cargo nextest run -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model` → 0 failures
- [x] I3 — `RUSTFLAGS="-D warnings" cargo clippy -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model -- -D warnings` → 0 warnings

### Anti-faking checks

- [x] AF1 — The `webglcontextlost` handler genuinely calls `prevent_default()` — omitting it would
      still compile and "look" correct while silently defeating the entire feature (browser gives up
      on restoration permanently)
- [x] AF2 — The lost-state check in `submit`/`output` happens strictly before any GL call in their
      bodies, not after — checking after would still call into an invalid context first

## Related Documentation

- `module/helper/tilemap_renderer/roadmap.md` — "webgl adapter gaps" section, bullet "WebGL context
  loss handling (`webglcontextlost` / `webglcontextrestored` events)" — the source of this task's
  scope
- `module/helper/tilemap_renderer/src/adapters/webgl.rs:282-330` — `WebGlBackend::new`, where
  listener registration is added
- `module/helper/tilemap_renderer/src/adapters/webgl.rs:1290-1360` — existing
  `Closure::once_into_js` async-image-load idiom this task's (persistent-variant) event registration
  follows
- `module/helper/tilemap_renderer/src/backend.rs:20-28,140-180` — `RenderError` enum and `Backend`
  trait this task extends
- `task/completed/064_tilemap_renderer_marker_resolution.md` — prior task that converted the
  adapter's other silent-skip families to loud signals; same crate/adapter, same "loud, not silent"
  convention this task's `RenderError::ContextLost` continues
- `task/decisions.md` — Q-02, the related-but-distinct `ImageSource::Encoded` decision this task
  deliberately does not resolve

## History

- **[2026-08-11]** `FILED` — Filed via `doc_tsk` while scoping `tilemap_renderer`'s remaining
  "webgl adapter gaps" (`roadmap.md`) for implementation. `ImageSource::Encoded` decoding was
  considered first but found blocked on an unowned wasm-binary-size decision (logged as
  `task/decisions.md` Q-02 instead of filed here); the other gaps (path/text/group rendering,
  gradient/pattern/clip-mask loading, effects, `BlendMode::Overlay`) are each independently
  substantial engineering efforts without a concrete committed need yet. Context-loss handling was
  chosen as the first webgl-gap task: unblocked, well-bounded, has a direct idiom precedent already
  in the file (`Closure::once_into_js`), and is a genuine production-readiness gap (browser tab
  backgrounding and GPU driver resets are common, not hypothetical) rather than a speculative
  rendering feature.
- **[2026-08-11]** `EXECUTED` — Implemented WebGL context-loss handling in `WebGlBackend`. Precisely
  scoped changes (see diff-contamination note below): `backend.rs` — added the `RenderError::ContextLost`
  unit variant (doc comment states the re-load-via-`load_assets` contract) and its `Display` arm.
  `Cargo.toml` — added `"EventTarget", "Event"` to the `web-sys` feature list (both previously absent
  from the workspace's dependency graph; confirmed against vendored `web-sys` 0.3.104 source, not
  guessed, before adding). `webgl.rs` — added a `context_lost : Rc<Cell<bool>>` field to `WebGlBackend`;
  added a `Self::register_context_loss_listeners(&gl, &context_lost)` call in `new()`; added the new
  `register_context_loss_listeners` method, registering persistent `webglcontextlost` (calls
  `prevent_default()`, sets the flag) and `webglcontextrestored` (clears the flag) listeners on the
  canvas resolved from `gl.canvas()`, both leaked via `.forget()` since context can be lost/restored
  more than once in a session (unlike the file's existing one-shot `Closure::once_into_js` async-
  image-load idiom); added a lost-state guard at the top of `submit()` and `output()`, each returning
  `RenderError::ContextLost` before issuing any GL call.
  **Fix during implementation:** the first compile attempt failed 3x with E0283 ("type annotations
  needed for `ScopedClosure<'_, _>`") because both closures only call `&self`-taking methods
  (`Cell::set`) on their captures, so the compiler couldn't pick between the `Fn`/`FnMut`
  `IntoWasmClosure` blanket impls. Fixed with an explicit `Closure::< dyn FnMut( web_sys::Event )
  >::new(...)` turbofish on both closures.
  **Diff-contamination note (read before verifying):** `git diff --stat` on the 3 touched files shows
  far more churn than the above — `Cargo.toml` also carries an unrelated `adapter-webgpu` feature
  block + `gpu_hal` dependency + a `wasm-bindgen-test` dev-dep; `backend.rs` lost an unrelated
  `#[allow(clippy::exhaustive_structs)]`; `webgl.rs` carries a large unrelated lint-hygiene/refactor
  pass (glob imports narrowed to explicit lists, `&[f32; 2]` params converted to by-value across
  several methods, several `#[allow(...)]`→`#[expect(..., reason = "...")]` conversions, a
  `load_geometry_sync`/`refresh_mesh_batch_vaos` extraction). None of this is from this task —
  confirmed via `stat` mtime (this task's own edits to these 3 files all postdate `started_at`
  15:41:40; the extra hunks predate it — e.g. `svg.rs`'s own unrelated diff, same pattern, has mtime
  12:00:38) and via content (thematically unrelated to context-loss). This is a live, shared,
  uncommitted working tree — same class of pre-existing-diff situation task 088's own History entry
  already documented for its M2 measurement. No content collision occurred (every edit landed via an
  exact-match `Edit` call, which errors loudly on a stale read), and every Test Matrix / Invariant
  check below ran against the full combined tree and passed — the two increments compose correctly.
  Verifier: scope C6/C9/M2 to the hunks described above (grep for
  `context_lost`/`ContextLost`/`register_context_loss_listeners`), not the raw `--stat` line counts.
  **Test Matrix results:**
  - T01 — `cargo check -p tilemap_renderer --target wasm32-unknown-unknown --features adapter-webgl`
    → exit 0 (`task/verified/-0025_longrun.log`)
  - T02 — `cargo check -p tilemap_renderer --no-default-features --features adapter-svg` → exit 0
    (`task/verified/-0026_longrun.log`)
  - T03 — `cargo check -p tilemap_renderer --no-default-features --features adapter-terminal` →
    exit 0 (same log)
  - T04 — `git diff --stat -- src/adapters/svg.rs src/adapters/terminal.rs`: `terminal.rs` empty
    (untouched, confirmed); `svg.rs` non-empty but pre-existing/unrelated per the contamination note
    above — this task's own diff touches neither file (same log)
  - T05 — `cargo doc -p tilemap_renderer --target wasm32-unknown-unknown --no-deps --features
    adapter-webgl` → exit 0, 3 pre-existing warnings unrelated to this change (`Transform`/
    `ResourceId` broken intra-doc links in `assets.rs`/`backend.rs`, one `commands.rs` code-block
    parse issue — none touching the new `ContextLost` doc comment). The task text's literal grep
    path (`tilemap_renderer/enum.RenderError.html`) doesn't exist — `mod_interface!`'s re-export
    puts the canonical page under `tilemap_renderer/backend/enum.RenderError.html` instead; grepping
    the correct path confirms the `ContextLost` variant's rendered doc text references re-calling
    `Backend::load_assets` (`task/verified/-0027_longrun.log`)
  - I2 — `cargo nextest run -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model`
    → 122/122 passed, 0 skipped (`task/verified/-0028_longrun.log`)
  - I3 — `RUSTFLAGS="-D warnings" cargo clippy -p tilemap_renderer --features adapter-svg,adapter-terminal,cli,scene-model -- -D warnings`
    → exit 0, 0 warnings (`task/verified/-0029_longrun.log`)
- **[2026-08-11]** `ACCEPTED` — Independent verifier (general-purpose Agent, blind dispatch) walked
  `§ Acceptance Verification : Procedure - Execution` against direct source inspection and its own
  re-execution of every Test Matrix row and Invariant — not the logged claims alone. Verdict: **ACCEPT**.
  All of C1–C9, M1–M2, I1–I3, AF1–AF2 independently confirmed true; re-run results: T01/I1 exit 0
  (fresh `longrun` recompile, not a cache hit), T02/T03 exit 0, I2 122/122 passed, I3 0 warnings, T05's
  `cargo doc` exit 0 with the corrected page path independently re-located and its `ContextLost` doc
  text independently re-read. Diff-contamination scoping (the EXECUTED entry's note above) verified
  accurate via direct content/mtime inspection of the actual diffs, not just trusted from the note.
  **Two non-blocking findings, left as-is (do not affect ACCEPT):** (1) the EXECUTED entry's T05
  citation (`-0027_longrun.log`) names a log that contains only the failed grep against the task text's
  literal (wrong) path — the successful grep against the corrected path was run directly, not captured
  in that log; the underlying claim is independently reconfirmed true regardless. (2) the diff-
  contamination note's `Cargo.toml` enumeration omits one unrelated line (`adapter-none = ["enabled"]`)
  — confirmed harmless, doesn't affect any AC or measurement, just an incomplete enumeration.
  (Verifier dispatch was interrupted mid-run by an unrelated harness restart and resumed via
  `SendMessage` from its preserved transcript — its findings above are from the completed, resumed run.)

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟡→🟢 | Adversarial: T05 was "read the doc comment" — not a runnable pass/fail command, violating the Delivery Requirements' own "every case backed by a test" bar | Replaced with a `cargo doc` build + `grep` on the rendered HTML — a genuine executable command with exit-code/match signal |
| D3 | Value / YAGNI | — | 🟢 | Adversarial: no live consumer exists today (weaker than 088/089's ADR-backed urgency) — but roadmap.md already tracks this as committed backlog, not speculative invention, and it's a real external-boundary concern (browser-triggered, not internal design choice); History section already discloses this reasoning transparently | — |
| D4 | Implementation Readiness | — | 🟢 | Adversarial: persistent-closure approach has no exact precedent in-file (only `once_into_js` exists) — but Goal text already honestly flags "confirmed against wasm-bindgen's own API at implementation time" rather than overclaiming | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | Adversarial: new `RenderError::ContextLost` variant touches a crate-shared enum — confirmed this is normal extensibility (like the existing 3 variants), not a new responsibility | — |
| **Total** | | — | 🟢 | 0 open | 1 fix |

**Verified by:** self (Tier 2 Dual-Role Self-Check) · **Date:** 2026-08-11

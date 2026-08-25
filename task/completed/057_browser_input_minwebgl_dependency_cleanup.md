# Replace browser_input's minwebgl dependency with ndarray_cg

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/browser_input
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

`browser_input` depends on `minwebgl` (workspace dep, `features = ["math"]`, optional, wired into the
`enabled` feature as `dep:minwebgl`) although it does no WebGL — an input-handling crate coupled to a
WebGL crate purely for utility re-exports. Replace it with the crates that own those utilities:

- **Math types:** `src/input.rs` uses `I32x2` and `F64x3`; `tests/active_pointers_test.rs:4` uses
  `minwebgl::math::I32x2`. Both types exist in `ndarray_cg` directly (`ndarray_cg/src/vector.rs` —
  `I32x2` line 33, `F64x3` line 26), which is where minwebgl's own math ultimately comes from.
- **`JsCast` trait (missed by the original note):** `src/util.rs:5` (`use gl::JsCast as _;`) and
  `src/input.rs:6` (`use min::{ JsCast as _, I32x2, F64x3 };`) import `JsCast` through minwebgl's
  re-export. `browser_input` has **no direct `wasm-bindgen` dependency**, so the swap must either add
  one or import via the existing `web-sys` dep's re-export (`web_sys::wasm_bindgen::JsCast`) — decide
  at pickup; prefer whichever the workspace's other non-GL browser crates already do.

Update `Cargo.toml` (dependency + `enabled`/`full` feature wiring), the three import sites, and verify:
minimum `cargo check --target wasm32-unknown-unknown -p browser_input` plus the crate's test suite per
workspace conventions (never set bare `RUSTFLAGS` on wasm32 builds — it clobbers
`.cargo/config.toml`'s `--cfg web_sys_unstable_apis`).

**Provenance:** migrated from `module/helper/browser_input/task/001_dependency_cleanup.md`, an
ungoverned pre-existing note retired by task 040 (its `001` filename also collided with this system's
task `001`; re-filing here under a fresh ID resolves that). The note's core claim (minwebgl used for
exactly two math types) was verified 2026-08-10 but found incomplete — the `JsCast` re-export and the
test-file import above are additional real coupling points its plan didn't cover.

## In Scope

- `module/helper/browser_input/Cargo.toml` — replace the `minwebgl` dependency with `ndarray_cg`;
  rewire the `enabled`/`full` feature lists; add the crate's own missing `Window`/`Document` `web-sys`
  features (the 4th coupling class, found only by the wasm32 build failing)
- `src/util.rs`, `src/input.rs` — swap `minwebgl`-sourced `I32x2`/`F64x3`/`JsCast` imports for
  `ndarray_cg`/`web_sys::wasm_bindgen::JsCast`
- `tests/active_pointers_test.rs` and `readme.md` — update the `minwebgl::math::I32x2` import and the
  Quick Start snippet to match

## Out of Scope

- Doc-comment cross-references to `minwebgl/src/texture/d2.rs` in `input.rs` (BUG-053 cfg-split
  citations) — left as documentation, not a dependency
- `tests/manual/readme.md`'s pointer to the `examples/minwebgl/touch_input_test` demo crate — a
  separate example crate path, left unchanged

## Verification

### Checklist

- [x] C1 — Is the `minwebgl` dependency genuinely removed from `Cargo.toml`, replaced by `ndarray_cg`? Current `[dependencies]` shows `ndarray_cg = { workspace = true, optional = true }` and `enabled = ["dep:ndarray_cg", "dep:web-sys", "dep:strum"]`; no `minwebgl` entry anywhere in the manifest.
- [x] C2 — Do the `web-sys` features include the claimed `Window`/`Document` additions (the 4th coupling class discovered mid-implementation)? Current manifest's `web-sys` feature list: `Window, Document, KeyboardEvent, PointerEvent, WheelEvent, Element, HtmlElement, CssStyleDeclaration` — both present.
- [x] C3 — Is `JsCast` now sourced via `web_sys::wasm_bindgen` rather than minwebgl's re-export, at both claimed sites? `grep -n "JsCast" src/util.rs src/input.rs` → `util.rs:4` and `input.rs:8` both import `wasm_bindgen::{ JsCast as _, ... }` through `web_sys`, zero through `minwebgl`.
- [x] C4 — Does the test file import `ndarray_cg::I32x2` instead of `minwebgl::math::I32x2`? `tests/active_pointers_test.rs:7` → `use ndarray_cg::I32x2;`.
- [x] C5 — Are the only residual `minwebgl` mentions in the crate the claimed documentation-only ones? `grep -in minwebgl src/*.rs tests/*.rs readme.md tests/manual/readme.md` → exactly 3 hits in `input.rs` (lines 203/216/229, doc comments citing `minwebgl/src/texture/d2.rs` re the BUG-053 cfg split) + 2 hits in `tests/manual/readme.md` (the `examples/minwebgl/touch_input_test` manual-test pointer) — no dependency-relevant hit.

### Measurements

- [x] M1 — `Cargo.toml` core dependency: `minwebgl` (was, `git show 4469eafb^:module/helper/browser_input/Cargo.toml`) → `ndarray_cg` (now) — `git diff 4469eafb^ 4469eafb -- module/helper/browser_input/Cargo.toml` shows the exact swap (`enabled = ["dep:minwebgl", ...]` → `["dep:ndarray_cg", ...]`; `minwebgl = { features = ["math"], ... }` → `ndarray_cg = { ... }`).
- [x] M2 — `web-sys` feature-list length: `6` (was, same diff/parent commit) → `8` (now, +`Window` +`Document`).

### Invariants

- [x] I1 — Test suite (crate-scoped): `cargo test -p browser_input --all-features` → exit 0; unittests 0/0, `active_pointers_test` 7/7, `pointer_type_test` 6/6, doc-tests 0/0.
- [x] I2 — Compiler/lints clean: `cargo clippy -p browser_input --all-targets --all-features -- -D warnings` → exit 0, zero warnings.
- [x] I3 — Primary target still builds (this task's actual point — WebGL decoupling without losing wasm32 buildability): `cargo check --target wasm32-unknown-unknown -p browser_input --all-features` → exit 0, 163s.

### Anti-faking checks

- [x] AF1 — Guards against `minwebgl` silently creeping back into `Cargo.toml` as a quick fix for a future missing type: re-run C1/C5's grep for `minwebgl` across `Cargo.toml` and `src/` — any hit outside the 3 documented `input.rs` doc-comment cross-references is a regression.
- [x] AF2 — Guards against the web-sys FEATURE-unification gap (the 4th coupling class this task discovered, invisible to any static grep — it only surfaced as a wasm32 compile failure) recurring on a future dependency edit: re-running I3 after any future edit to `browser_input`'s `web-sys` feature list or dependency graph is the only real guard, since a transitively-supplied feature silently disappearing cannot be caught by source-level diffing alone.

## History

- **[2026-08-10]** `FILED` — Migrated from browser_input's informal `task/` note by task 040
  (disposition option b: adopt the idea into the root system, retire the note). Technical claims
  re-verified against current source at filing time; two coupling points the note missed are recorded
  in the Goal.
- **[2026-08-10]** `IMPLEMENTED` — Census at pickup confirmed the Goal's three import sites exactly
  (`src/util.rs:4-5`, `src/input.rs:5-6`, `tests/active_pointers_test.rs:4`) plus one the Goal
  missed: the readme's Quick Start snippet (`use minwebgl as gl; ... use gl::JsCast as _;`).
  Type identity proven before touching anything: minwebgl's `math` feature = `mingl/math` =
  `dep:ndarray_cg` with `reuse ::ndarray_cg` (mingl/src/math.rs:10), and `I32x2`/`F64x3` are
  `exposed use` at ndarray_cg's root (vector.rs) — so `minwebgl::math::I32x2` IS
  `ndarray_cg::I32x2` (same `Vector< i32, 2 >` alias) and the swap is invisible to dependents.
  JsCast decision (the Goal's decide-at-pickup point): `web_sys::wasm_bindgen::JsCast` — the
  in-crate precedent (both source files already imported `web_sys::wasm_bindgen::prelude::Closure`);
  no other non-GL browser crate uses JsCast at all (grep browser_log/animation: zero hits); no new
  `wasm-bindgen` dependency added. Changes: Cargo.toml dep swap (`ndarray_cg = { workspace = true,
  optional = true }`, mirroring mingl's declaration) + `enabled` rewire (`dep:minwebgl` →
  `dep:ndarray_cg`); the three import sites; the readme snippet. The first verification launch
  then exposed a FOURTH coupling class the Goal's census never enumerated: web-sys FEATURE
  unification — browser_input's own web-sys feature list lacked `Window` and `Document`, silently
  completed by minwebgl's transitive web-sys features; with minwebgl gone, `web_sys::window()`
  failed to resolve at both call sites (log `-0048` exit 101). Fixed by adding `"Window"`,
  `"Document"` to browser_input's own web-sys features (the crate's other needs — `Event`,
  `EventTarget`, `Node` — arrive transitively via its declared `PointerEvent`/`Element` chains).
  The `minwebgl/src/texture/d2.rs` mentions at input.rs:183/196/209 are documentation
  cross-references (where the `web_sys_unstable_apis` type split is visible), not dependencies —
  kept; tests/manual/readme.md's pointer to `examples/minwebgl/touch_input_test` is a demo-crate
  path, likewise kept.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Log `-0049` (`cargo check
  --target wasm32-unknown-unknown -p browser_input --all-features && cargo test -p browser_input
  --all-features`) exit 0, 26s: wasm32 check clean, native suite active_pointers 7/7 +
  pointer_type 6/6 + doc 0. Blast-radius spot-check log `-0051`: `cargo check --target
  wasm32-unknown-unknown -p minwebgl_touch_input_test` (the canonical browser_input dependent,
  compiling browser_input + minwebgl together) exit 0, 19s — the exact dependent class the
  feature-unification failure could have hit. `grep -rn minwebgl browser_input/` residue is
  documentation-only (the three source cross-reference comments + the manual-testing example
  path). Never set bare `RUSTFLAGS` on the wasm32 builds per the Goal's warning — none was set.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | All edits within browser_input + task/ + health.md; input.rs doc cross-references and manual-test example path deliberately kept (not coupling) | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | Goal's census re-verified at pickup: 3 import sites exact; JsCast decide-at-pickup point resolved by in-crate precedent | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | No new wasm-bindgen dependency added — JsCast reached through the already-present web-sys re-export; ndarray_cg declared exactly as mingl does | — |
| D4 | Implementation Readiness | 🟡 | 🟢 | Confirming pass took the Goal's 3-site census as complete; adversarial pass (residual grep + running the wasm32 gate first) found two missed coupling points: the readme Quick Start still importing minwebgl, and web-sys FEATURE unification — browser_input's own feature list lacked Window/Document, silently completed by minwebgl's transitive features (log `-0048` exit 101, E0425 `window` ×2) | Readme snippet decoupled; `"Window"`, `"Document"` added to browser_input's own web-sys features |
| D5 | Execution Scope | 🟢 | 🟢 | 6 files touched: Cargo.toml, util.rs, input.rs, active_pointers_test.rs, readme.md, this record | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Single-crate task; type identity proven (minwebgl math = mingl math = reuse ndarray_cg) so no dependent crate is affected | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Input crate no longer coupled to a WebGL crate for utility re-exports — the task's own point | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | No bare RUSTFLAGS on wasm32 builds (Goal's warning heeded); house style on all authored lines; Edit tool on existing files | — |
| B2 | Test-First | 🟢 | 🟢 | No behavior change intended — existing suite is the regression net; test import swapped in the same change | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | Genuine red run this task: log `-0048` exit 101 — the hidden feature coupling was real, not hypothetical; fix verified green in `-0049` | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Feature gap fixed by declaring browser_input's own needs, not by re-adding a GL crate or over-declaring (Event/EventTarget/Node left transitive via declared chains) | — |
| B5 | Fix Verification | 🟢 | 🟢 | Log `-0049` exit 0: wasm32 check + active_pointers 7/7 + pointer_type 6/6 + doc 0; log `-0051` exit 0: dependent example minwebgl_touch_input_test wasm32 check green | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Feature-unification coupling class recorded in IMPLEMENTED (the census-invisible failure mode); JsCast decision + rejected alternative recorded | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | Net-negative diff in src (4 import lines → 2); no backup files; residual minwebgl mentions are documentation-only, each justified in the record | — |
| **Total** | | 🔴 | 🟢 | 1 finding (two missed coupling points) resolved in-loop | 15/15 |

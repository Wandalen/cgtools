# BUG-279: `minwebgl`'s `enabled` feature doesn't declare its real dependency on `mingl/web` and `mingl/web_log`, breaking `--features enabled` alone

- **Severity:** Medium (no runtime defect -- a compile-time feature-graph gap that breaks any
  consumer selecting `enabled` without also separately selecting both `mingl/web` and
  `mingl/web_log`)
- **state:** Completed
- **Affects:** `minwebgl`'s `enabled` Cargo feature (`Cargo.toml`); `src/exec_loop.rs`'s
  unconditional `reuse ::mingl::web::exec_loop;` and `src/log.rs`'s unconditional
  `reuse ::mingl::web::log;`
- **Component:** `module/min/minwebgl` (`Cargo.toml`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`minwebgl`'s `lib.rs` compiles `layer exec_loop;` and `layer log;` unconditionally -- neither
carries any `#[cfg(...)]` gate at all, unlike every other layer in the file (`web`, `future`,
`model`/`file`, `math`, `diagnostics` are all feature-gated). Both layers' bodies are thin
`mod_interface!` reuse wrappers: `src/exec_loop.rs` is `reuse ::mingl::web::exec_loop;` and
`src/log.rs` is `reuse ::mingl::web::log;`. But `mingl::web` itself is gated behind
`#[cfg(feature = "web")]` in `mingl/src/lib.rs`, and `mingl::web::log` is additionally gated
behind a *nested* `#[cfg(feature = "web_log")]` inside `mingl/src/web.rs`. `minwebgl`'s own
`enabled` feature -- meant as this crate's baseline, always-active feature -- never forwarded
either `mingl/web` or `mingl/web_log`. Selecting `enabled` without also separately selecting
`mingl`'s `web`/`web_log` (or `minwebgl`'s own `web`/`log` features, which forward them) fails to
compile with `E0433: cannot find 'web' in 'mingl'` the moment `exec_loop.rs`/`log.rs` try to
resolve their `reuse` targets.

## Impact

**Who is affected:** any consumer selecting `minwebgl`'s `enabled` feature in isolation (or via
any combination that omits `web`/`log`) without happening to also request them separately. The
crate's own `default` feature bundle (`enabled + constants + diagnostics + web + future + file +
log`) always carries `web`+`log` alongside `enabled`, and so does `--all-features` -- no real
default-feature or full-feature consumer had ever hit the gap before this review.

**What breaks:** `cargo check -p minwebgl --no-default-features --features enabled` (and any
equivalent invocation, including `cargo build`/`cargo test` under the same feature selection)
fails outright with a compile error, not a runtime defect. `enabled` already hard-requires the
exact same underlying dependencies (`dep:web-sys`, `dep:wasm-bindgen`,
`dep:wasm-bindgen-futures`, `dep:js-sys`) that `mingl/web` itself gates, so the crate was never
meaningfully usable without them regardless -- the feature graph just never said so explicitly.

**Entity Scope:** `None` -- Cargo feature-graph defect, not entity directory instances.

## How Discovered

During this session's review of `minwebgl`'s exec-loop/geometry/model/program layer (13 files:
`exec_loop.rs`, `file.rs`, `future.rs`, `geometry.rs`, `index.rs`, `lib.rs`, `log.rs`, `math.rs`,
`mem.rs`, `model.rs`, `model/obj.rs`, `panic.rs`, `program.rs`), most files turned out to be thin
`mod_interface!` reuse wrappers with no independent logic of their own to break. Per this
session's own precedent (BUG-270's missing-feature-dependency class, explicitly flagged as a
domain hint for this review), the fork additionally spot-checked isolated feature combinations
as an adversarial check beyond per-file inspection: `cargo check -p minwebgl
--no-default-features --features enabled` failed to compile, tracing directly back to `lib.rs`'s
two unconditional layers. Independently re-verified via `grep` of `lib.rs`'s per-layer `#[cfg]`
gates and `mingl/src/lib.rs`'s/`src/web.rs`'s own gates before accepting the finding.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo check -p minwebgl --no-default-features --features enabled
```
**Expected** (fixed): compiles (4 unrelated, pre-existing errors remain from `texture/d2.rs` and
`drawbuffers.rs` -- out of this bug's scope, see Verification).
**Actual** (pre-fix, confirmed via temporary manual removal of just the fix's two added lines --
see Verification for why a whole-file `git stash` was avoided):
```
error[E0433]: cannot find `web` in `mingl`
  --> module/min/minwebgl/src/exec_loop.rs:10:23
error[E0433]: cannot find `web` in `mingl`
  --> module/min/minwebgl/src/log.rs:10:23
```

## Root Cause

`Cargo.toml` (pre-fix):
```toml
enabled = [
  "dep:mingl",
  "dep:asbytes",
  "dep:browser_log",
  "dep:wasm-bindgen-futures",
  "dep:wasm-bindgen",
  "dep:js-sys",
  "dep:web-sys",
]
```
`src/lib.rs` (unchanged, both pre- and post-fix):
```rust
crate::mod_interface!
{
  layer exec_loop;
  // ...
  layer log;
  // ...
  #[ cfg( feature = "web" ) ]
  layer web;
}
```
`exec_loop.rs`/`log.rs` (unchanged): thin `reuse ::mingl::web::exec_loop;` /
`reuse ::mingl::web::log;` wrappers, no `#[cfg(...)]` of their own. `mingl/src/lib.rs` gates its
own `layer web;` behind `#[cfg(feature = "web")]`; `mingl/src/web.rs` additionally gates `layer
log;` (nested one level deeper) behind `#[cfg(feature = "web_log")]`. `enabled`'s dependency list
never forwarded either gate, despite `exec_loop`/`log` being compiled unconditionally whenever
`enabled` is selected at all.

**Two-stage discovery:** adding only `mingl/web` to `enabled` cleared the first error but
surfaced a second, nested one -- `E0433: cannot find 'log' in 'web'` at `src/log.rs:10` -- because
`mingl::web::log` carries its own separate `web_log` gate inside the now-resolved `mingl::web`
module. Both `mingl/web` and `mingl/web_log` were required to fully resolve the gap.

## Why Not Caught

`web`, `log`, `future`, and `file` are all bundled together in this crate's own `default`
feature set, and every pre-existing test invocation ran via `--all-features` or plain `cargo
test` (default features) -- both always carry `web`+`log` alongside `enabled`, so nothing had
ever selected `enabled` without them until this session's isolated-feature spot check -- the same
isolated-feature-combination technique that found the sibling BUG-274 (`diagnostics` omitting
`future`/`file`) in this same `Cargo.toml`.

## Fix Applied (2026-08-17)

**`Cargo.toml`:** changed `enabled`'s Cargo feature list to additionally include `"mingl/web"`
and `"mingl/web_log"`, making the feature graph match what `src/exec_loop.rs` and `src/log.rs`
actually, unconditionally need. No source file changed -- both files' `reuse` targets were
already correct; only the feature declaration was incomplete.

**`tests/enabled_feature_web_gate_test.rs`** (new test):
`exec_loop_and_log_modules_resolve_under_enabled_feature` references
`minwebgl::exec_loop::run::<fn(f64) -> bool>` (function-pointer only, never invoked -- it starts a
real `requestAnimationFrame` loop and hangs/panics outside a live browser `window`) and
`minwebgl::log::Level::Info` (pure data, no browser console touched), proving both modules resolve
through their full `reuse` chains under the crate's standard `--all-features` test run. A single
feature-selection test binary cannot itself reproduce the "fails under `enabled` alone / passes
under `--all-features`" contrast; that authoritative before/after proof lives in this report's
Verification section instead (same convention as the sibling BUG-274 test).

## Verification

`longrun`-detached, from repo root:
- **Pre-fix (RED):** temporarily removed just the two added lines (`"mingl/web"`,
  `"mingl/web_log"`) from `Cargo.toml`'s `enabled` list via a surgical manual edit -- not a
  whole-file `git stash`, since the file also carries the unrelated, already-applied BUG-274 fix
  (`diagnostics` requiring `future`/`file`), and stashing the entire file would have reverted that
  fix's hunk too. `cargo check -p minwebgl --no-default-features --features enabled`: fails,
  `error[E0433]: cannot find 'web' in 'mingl'` at both `src/exec_loop.rs:10` and `src/log.rs:10`,
  exactly as diagnosed.
- **Nested-gate check:** with only `"mingl/web"` restored (not yet `"mingl/web_log"`): still
  fails, `error[E0433]: cannot find 'log' in 'web'` at `src/log.rs:10` -- confirmed the second,
  nested gate before finalizing the fix.
- **Post-fix (GREEN, both lines restored):** `cargo check -p minwebgl --no-default-features
  --features enabled`: 4 errors remain (`texture/d2.rs`'s `JsFuture` import, `drawbuffers.rs`'s
  `gl::NONE` x2 and `gl::COLOR_ATTACHMENT0`) -- all pre-existing, unrelated to this bug, in files
  outside this review's scope. Zero `exec_loop.rs`/`log.rs` errors remain.
- `cargo test -p minwebgl --all-features`: 17 passed / 0 failed across 8 integration test
  binaries (`clean_test` 2, `data_type_test` 2, `diagnostics_test` 1, `drawbuffers_test` 2,
  `enabled_feature_web_gate_test` 1 [new], `geometry_test` 2, `sprite_upload_test` 5,
  `uniform_test` 2), plus 1 doctest passed (7 ignored -- browser/wasm-context-only doctests,
  expected).
- `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings`: clean, exit 0 (after
  fixing the new test's own `no_effect_underscore_binding` lint by using anonymous `_` bindings
  instead of named `_run`/`_level`).

## Generalized Version

**Broken assumption:** a `layer` with no per-item `#[cfg(...)]` reads as "always available," but
if its body `reuse`s from an optionally-gated path in another crate, it silently inherits that
crate's gate as a real (but unstated) prerequisite of whichever outer feature always compiles it.
Gates can also nest -- a feature-gated module can itself contain a further feature-gated item one
level deeper -- so resolving the first gate found isn't sufficient; the fix must be verified
against a clean, isolated re-check rather than assumed complete after the first error clears.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during this session's review of `minwebgl`'s exec-loop/geometry/model/program layer (13 assigned files), via an adversarial isolated-feature-combination spot check beyond per-file inspection, prompted by this session's own BUG-270/BUG-274 precedent for missing feature-gate dependencies in this same crate's `Cargo.toml`. Root cause: `enabled` omitted its real, two-level-nested dependency on `mingl/web` and `mingl/web_log`, which `src/exec_loop.rs`/`src/log.rs` use unconditionally with no `#[cfg(...)]` guard of their own. Fixed by adding both to `enabled`'s feature-requirement list. Verified via 1 new native test (confirmed fail pre-fix / pass post-fix via temporary manual two-line revert-and-rerun, avoiding a whole-file `git stash` that would have also reverted the unrelated, already-applied BUG-274 fix in the same file) plus the full `--all-features` suite (17/17 + 1 doctest) and clean clippy. Originally scanned and intended to file as BUG-275, but a fresh on-disk scan immediately before writing this report found BUG-275 had, in the interim, already been independently claimed and closed by a different concurrent fork (`minwebgpu`'s storage-texture-binding-layout default-format bug, unrelated) -- along with BUG-276, BUG-277, and BUG-278, all claimed by other concurrent forks in the same window. Re-scanned and filed as BUG-279 after confirming 279 through at least 299 were genuinely unclaimed (`task/readme.md`'s `highest_id` stood at 278 at filing time), matching this session's established collision-recovery precedent (see e.g. BUG-270's own filing, and `task/readme.md`'s documented ID-namespace-collision history). |

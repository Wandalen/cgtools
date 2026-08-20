# BUG-229: `DebugLog`'s trait-default methods tag every `Record` with their own module path, not the real caller's, silently defeating `Config::target_filter`

- **Severity:** Medium (no panic/crash/data-corruption; a silent complete-output-loss defect
  in an opt-in feature -- `Config::target_filter`, the crate's own advertised primary use case
  -- rather than a break in the zero-config default path, which remains unaffected since an
  unset filter always returns `true` regardless of `target`)
- **state:** Completed
- **Affects:** Every `DebugLog` consumer that sets `Config::target_filter` (the crate's own
  documented example: `Config::default().target_filter( "lib_name" )`) -- for those consumers,
  every `debug_trace`/`debug_info`/`debug_warn`/`debug_error`/`debug_log` call is silently
  dropped, since the mistagged target never starts with their configured prefix. 3 real call
  sites found workspace-wide: `examples/minwebgl/attributes_matrix`, `examples/minwebgl/
  uniforms_ubo` (2 calls), and this crate's own `tests/debug_log_test.rs` (5 calls) -- none of
  which currently set `target_filter`, so none were silently broken in practice, but all 3 use
  the API in a way any future `target_filter` adoption would immediately hit.
- **Component:** `module/helper/browser_log` (`src/log/debug_log.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** Same file/trait as BUG-167 (file/line lexical resolution) -- shares the root
  cause CLASS (macro lexical resolution inside a trait-default body) but an independent defect:
  BUG-167's own fix comment (still present in this file) explicitly notes `module_path!()` was
  NOT addressed by that fix, since `#[track_caller]` only helps `file!()`/`line!()`.

## Symptom

```rust
// pre-fix -- DebugLog::debug_log, every call, from any external crate
.module_path( Some( module_path!() ) )   // ALWAYS "browser_log::log::debug_log::private"
.target( module_path!() )                // ALWAYS "browser_log::log::debug_log::private"
```

Every `Record` emitted through any `DebugLog` convenience method is tagged with `target`/
`module_path` equal to `browser_log`'s own internal trait-defining module -- never the real
external caller's module -- regardless of which crate or function actually called it.

## Impact

**Who is affected:** Any consumer setting `Config::target_filter` -- the crate's own documented
primary use case (`readme.md`/doc-comment example: filtering to `"lib_name"`).

**What breaks:** `BrowserLogger::enabled()` (`src/log/setup.rs`) gates every record on
`metadata.target().starts_with(prefix)`. Since `target` is always `"browser_log::log::
debug_log::private"` regardless of the true caller, it never starts with any consumer's own
crate-name prefix -- every `DebugLog`-originated record is silently dropped, with no error,
warning, or other visible signal. The zero-config default (no `target_filter` set) is
unaffected, since `enabled()` falls back to `true` in that case.

**Magnitude:** 1 trait (`DebugLog`), 5 methods, all sharing the same 2 mis-resolved builder
calls inside `debug_log`'s single default-method body.

**Entity Scope:** None -- a code-level defect.

## How Discovered

This session's scouting pass of `browser_log` (previously unaudited beyond BUG-167's own
investigation), reading `debug_log.rs` in full and noting BUG-167's own fix comment explicitly
flagging `module_path!()` as an unaddressed gap.

## Minimum Reproducible Example

```rust
let sample = Sample { value : 7 };
let this_module = module_path!();       // e.g. "debug_log_test", the REAL caller's module
sample.debug_info( this_module );
// pre-fix: captured Record's `target()`/`module_path()` == "browser_log::log::debug_log::private"
// post-fix: captured Record's `target()`/`module_path()` == this_module
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/browser_log && cargo nextest run --all-features -E 'test(debug_log_methods_report_the_real_caller_location_and_module)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `module_path!()`, called inside `DebugLog`'s trait-default method body, is lexical and always resolves to this trait's own defining module, regardless of the real external caller. | ✅ Root Cause | Direct read of pre-fix `debug_log.rs` shows both `.module_path()`/`.target()` builder calls using `module_path!()` written inside the trait body itself; confirmed empirically via temporary-revert-and-rerun producing the exact value `"browser_log::log::debug_log::private"`. | E1, E3 |
| H2 | `#[track_caller]` (already applied to these methods for BUG-167) also fixes `module_path!()`'s resolution, the same way it fixed `file!()`/`line!()`. | ❌ Falsified | `#[track_caller]` only affects what `std::panic::Location::caller()` returns -- it has no effect on `module_path!()`, which remains purely lexical with no dynamic/caller-tracking equivalent in stable Rust. BUG-167's own fix comment, still present in this file pre-this-fix, explicitly documents this gap. | E2, E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/browser_log/src/log/debug_log.rs`, `DebugLog::debug_log` (pre-fix, direct read) | `module_path!()` used inside the trait's own default body for both `.module_path()` and `.target()` builder calls. | H1 ✅ |
| E2 | `module/helper/browser_log/src/log/debug_log.rs`, `Fix(BUG-167)` comment (pre-fix, direct read) | Explicitly documents that the BUG-167 fix addressed `file!()`/`line!()` via `#[track_caller]` but did not address `module_path!()`. | H2 ❌ |
| E3 | Temporary direct-source-edit revert-and-rerun (this fix) | Reverting `.module_path()`/`.target()` back to internal `module_path!()` calls produced test failure `left: "browser_log::log::debug_log::private", right: "debug_log_test"` -- an exact, unambiguous empirical confirmation. | H1 ✅ |
| E4 | `module/helper/browser_log/src/log/setup.rs`, `BrowserLogger::enabled()` (direct read) | `target_filter` is matched via `metadata.target().starts_with(prefix)` -- confirms a permanently-mis-tagged `target` silently defeats this filter for every consumer who sets one. | H1 ✅ |
| E5 | `~/.cargo/registry/.../log-0.4.33/src/macros.rs` lines 403-405 (external dependency, direct read) | Confirms `log_enabled!(target: $target:expr, $lvl:expr)` is valid, supported macro syntax in the pinned `log` version -- validates the fix's mechanism (mirroring `log`'s own escape hatch for this exact limitation). | Fix design |

## Root Cause

`module_path!()`, exactly like `file!()`/`line!()`, is a lexical macro resolved at its own
write-site -- inside `DebugLog`'s trait-default method body, that site is always this trait's
own defining module, never the real external caller's, regardless of `#[track_caller]`,
`#[inline]`, or the blanket `impl<T> DebugLog for T` covering every `fmt::Debug` type. Unlike
`file!()`/`line!()`, there is no stable `Location`-style dynamic equivalent for module path in
Rust, so the only correct fix is to require the caller to supply it explicitly -- mirroring
`log`'s own `log!(target: "...", ...)` / `log_enabled!(target: "...", ...)` escape hatch for
this identical, well-known limitation.

## Why Not Caught

BUG-167's own regression test (`tests/debug_log_test.rs`) only ever asserted on `file()`/
`line()`/`args()`; it captured `Record`s without ever inspecting `target()`/`module_path()` at
all, so this defect shipped underneath a fully green, specifically-targeted regression suite for
the exact same trait and the exact same class of lexical-resolution bug.

## Fix Location

`module/helper/browser_log/src/log/debug_log.rs`: `debug_log`/`debug_trace`/`debug_info`/
`debug_warn`/`debug_error` all gained an explicit `target : &str` parameter. Internally,
`log::log_enabled!( target: target, level )` replaces the bare-level pre-filter check, and
`.module_path( Some( target ) )`/`.target( target )` replace the two internal `module_path!()`
uses. This is a breaking API change (acceptable per project convention; no backward-compat
shim added) -- all 3 real call sites found workspace-wide were updated to pass their own
`module_path!()`: `examples/minwebgl/attributes_matrix/src/main.rs:134`, `examples/minwebgl/
uniforms_ubo/src/main.rs:54-55`, and `module/helper/browser_log/tests/debug_log_test.rs` (5
calls, extended in place rather than duplicated -- see Prevention).

**Scouting-methodology note:** the earlier scouting pass's `grep -rn "browser_log::"` reported
zero external consumers -- incomplete, because these methods are reached via the blanket
`impl<T> DebugLog for T` through a plain `use browser_log::DebugLog;` (or `minwebgl`'s
re-exporting prelude) import, so call sites read `.debug_info()` with no `browser_log::` path
prefix anywhere in the calling line. Verified instead via `grep -rnE
"\.debug_trace\(|\.debug_info\(|\.debug_warn\(|\.debug_error\(|\.debug_log\("` across the whole
workspace, which found all 3 real call sites.

## Prevention

`tests/debug_log_test.rs`'s existing BUG-167 test was extended (not duplicated -- `log::
set_logger` is process-global and callable once per test binary, so a second competing `#[
test ]` installing its own logger was not an option) to also capture and assert on `target()`/
`module_path()` for all 5 methods, not just `file()`/`line()`/`args()` as before.

## Pitfall

A regression test scoped to "this trait's caller-reporting correctness" that asserts on only
some of a `Record`'s caller-reporting fields (file, line) while silently leaving a sibling field
(target, module_path) of the exact same lexical-resolution defect class completely unchecked,
lets that sibling field's bug ship invisibly right next to the one that was actually fixed --
until directly audited. Separately: a "zero consumers" scouting conclusion based on grepping
for a crate's own `crate_name::` path prefix systematically undercounts real usage of any
blanket-impl trait method, which is called with no such prefix anywhere at the call site once a
`use` import is in scope -- verifying "zero consumers" for a trait-shaped API requires grepping
for the method names themselves, not the defining crate's path.

## Generalized Version

**Broken assumption:** "a crate with zero direct `crate_name::` path references anywhere else
in the workspace has zero external consumers."

**Confirmed general rule:** A blanket trait impl (`impl<T> Trait for T where T : Bound {}`) is
invoked via bare method syntax (`.method()`) with no `crate_name::` prefix anywhere at the call
site once a `use crate_name::Trait;` (or a re-exporting prelude) is in scope --
`grep -rn "crate_name::"` cannot find these call sites. Verifying "zero consumers" for a
trait-shaped API requires grepping for the trait's method names (or the trait name via `use`),
never just the crate's own path prefix.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `browser_log` scouting pass; independently confirmed via direct reads of `debug_log.rs`/`setup.rs`, and of the pinned `log` crate's own macro source to validate the fix's mechanism. |
| 2026-08-17 | fixed | Added explicit `target : &str` param to `debug_log` and its 4 convenience methods; callers now pass their own `module_path!()`. Updated all 3 real call sites found workspace-wide via a method-name grep (wider than the initial scout's `browser_log::`-prefix grep, which missed all of them). |
| 2026-08-17 | verified | `cargo nextest run -p browser_log --all-features`: 9/9 passed, 0 skipped. `cargo test --doc -p browser_log --all-features`: clean. `cargo clippy -p browser_log --all-targets --all-features -- -D warnings`: clean. `cargo clippy -p minwebgl_attributes_matrix -p minwebgl_uniforms_ubo --target wasm32-unknown-unknown --all-targets -- -D warnings`: clean. Fix verified via temporary direct-source-edit revert-and-rerun (`left: "browser_log::log::debug_log::private", right: "debug_log_test"` pre-fix, passed post-fix). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Confirming pass: extended-test MRE is a deterministic `assert_eq!` string comparison, no timing dependency, so no coincidental-pass risk (unlike BUG-228's sawtooth concern). Adversarial pass: checked whether a caller passing a wrong/empty `target` string could mask the fix -- no, the trait cannot validate a caller-supplied string any more than `log`'s own `target:` macro form does; this matches the upstream crate's own accepted design, not a gap in this fix. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly distinguished from BUG-167 (file/line fix, explicitly left `module_path!()` unaddressed per its own fix comment) as an independent defect sharing only the root-cause class. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct source reads (`debug_log.rs`, `setup.rs`), an external dependency source read (pinned `log 0.4.33` macro definitions) validating the fix mechanism, and empirical revert-rerun proof producing the exact mis-tagged value. | — |
| D5 | Execution Scope | — | 🟢 | Confirming pass: fix confined to adding a `target` param plus updating the 2 internal `module_path!()` call sites. Adversarial pass: grepped the entire workspace for `module_path!()` post-fix to confirm no other instance of this defect pattern remains anywhere (found only the fix's own doc comments and correct caller-supplied usages). | — |
| D6 | Crate Scope Unity | — | 🟢 | Confirming pass: breaking signature change required updating 2 downstream consumer crates. Adversarial pass: re-grepped for all 5 method names (not just the `debug_info` calls that turned up) across the entire workspace via one combined regex, confirming no other call site was missed; both updated call sites additionally verified against the `wasm32-unknown-unknown` target (their real compilation target), not just native. | — |

**Reproduced:** Confirmed via `cargo nextest` (fail pre-fix with exact value
`"browser_log::log::debug_log::private"`, pass post-fix) and temporary direct-source-edit
revert-and-rerun. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/browser_log/src/log/debug_log.rs` | `DebugLog::debug_log`/`debug_trace`/`debug_info`/`debug_warn`/`debug_error`: added explicit `target : &str` parameter; internal `module_path!()` uses replaced with the caller-supplied `target` (full `Fix(BUG-229)` comment block). |
| `examples/minwebgl/attributes_matrix/src/main.rs` | Updated 1 `.debug_info()` call site to pass `module_path!()` for the new required `target` parameter. |
| `examples/minwebgl/uniforms_ubo/src/main.rs` | Updated 2 `.debug_info()` call sites to pass `module_path!()` for the new required `target` parameter. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/browser_log/tests/debug_log_test.rs` | Extended the existing BUG-167 test (renamed `debug_log_methods_report_the_real_caller_location_and_module`) to also capture and assert `target()`/`module_path()` on all 5 methods (`bug_reproducer(BUG-167)`, `bug_reproducer(BUG-229)`); updated all 5 call sites for the new `target` parameter. |

## Refs: docs/

| File | Change |
|------|--------|
| — | None — no pre-existing doc section described this defect. |

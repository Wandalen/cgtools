# Fix browser_log's false feature claims in docs

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/browser_log
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

`browser_log`'s documentation claims a feature or capability the audit found not actually implemented in
the crate's real source (P5 — remaining doc drift, Fix-in-place). Note: this session's git status shows
`module/helper/browser_log/Cargo.toml` as modified (uncommitted), so check whether that in-flight change
already touches the claim in question before starting. **Exact claim and file were not preserved precisely
through this session's context compaction — re-derive by diffing the crate's readme/doc claims against
`src/` at pickup.** Kept as a separate task from task 030 (mingl's own false claims) per Crate Scope
Unity even though both were found in the same audit pass.

## In Scope

- `readme.md` — reworded 4 false claims ("timestamps" in Formatted Output, "Performance Logging" feature,
  "all WebAssembly runtimes" compatibility, redundant `console_error_panic_hook` recommendation) to match
  actual source behavior
- `readme.md`'s API Reference — added the previously-omitted `setup()` function and `DebugLog` trait

## Out of Scope

- `changelog.md`'s 0.3.0 entry — left as a historical record, not rewritten
- Duplicate `licence`/`license` files — noted but not resolved, surfaced for user decision
- `mingl`'s false claims — filed separately as task 030 per Crate Scope Unity

## Verification

### Checklist

- [x] C1 — Is the false "timestamps" claim gone from `readme.md`'s Formatted Output bullet? `grep -c "timestamp" readme.md` → `0` (was 1 hit pre-fix: "Structured log messages with timestamps and context", `git show 25ceae76:module/helper/browser_log/readme.md`); current wording (readme.md:12) reads "CSS-styled messages with a level badge and `file:line` source context", matching the real format built in `src/log/setup.rs`.
- [x] C2 — Is the false "Performance Logging" feature bullet replaced with an honest re-export description? `grep -c "Performance Logging" readme.md` → `0` (1 hit pre-fix); readme.md:13 now reads "**Console Re-export** - `browser_log::log::console` re-exports `web_sys::console`..." — `console` is confirmed a plain re-export via `exposed use ::web_sys::console;` in `src/log/mod.rs:42`, not a first-party timing feature.
- [x] C3 — Is the false "all WebAssembly runtimes" compatibility claim reworded to name the real requirement? `grep -c "all WebAssembly runtimes" readme.md` → `0` (1 hit pre-fix); readme.md:254 now reads "Requires a JavaScript-hosted runtime for console output (browsers, Node.js, Deno) — not WASI runtimes", matching `src/panic.rs`'s `wasm_bindgen` externs (`console.error`, `Error` — both JS-host-only bindings).
- [x] C4 — Is the redundant/misleading `console_error_panic_hook = "0.1"` wasm-pack recommendation removed? `grep -c 'console_error_panic_hook = "0.1"' readme.md` → `0` (1 hit pre-fix); readme.md:243-244 now states "No separate `console_error_panic_hook` dependency is needed — `browser_log::panic` provides the console panic hook... itself."
- [x] C5 — Does the API Reference now list the top-level `setup()` function and the `DebugLog` trait that were previously omitted? Both present in the Core Functions table (readme.md:92,95); cross-checked against source — `pub fn setup(config: Config)` exists at `src/lib.rs:24` (combines `panic::setup` + `log::setup::setup`), and `pub trait DebugLog` with `debug_info()`/`debug_trace()`/`debug_warn()`/`debug_error()` exists at `src/log/debug_log.rs:16-52`.
- [x] C6 — Are the two explicitly-noted-as-NOT-fixed items still genuinely untouched? `changelog.md`'s 0.3.0 entry still reads "Structured logging with configurable output formats" (changelog.md:16, left as historical record, as claimed). The licence/license duplication note is now **stale**: only `license` exists (`licence` is gone) — `git log --diff-filter=D -- module/helper/browser_log/licence` shows it was deleted by a later, unrelated commit `5f33be66` ("feat: consolidate test infrastructure and refactor module architecture"), not by this task. This doesn't contradict task 029's claim (it correctly said "out of scope, not touched", and never did); the pre-existing condition it observed simply no longer exists, resolved by unrelated later work.

### Measurements

- [x] M1 — Fictional-claim phrase count in `readme.md` (4 tracked phrases: "timestamp", "Performance Logging", "all WebAssembly runtimes", `console_error_panic_hook = "0.1"`): `0` (was: `4`/4 present, `git show 25ceae76:module/helper/browser_log/readme.md`).

### Invariants

- [x] I1 — Test suite (crate-scoped, includes the doctested readme): `cargo nextest run -p browser_log --all-features && cargo test -p browser_log --doc --all-features` → exit 0; nextest 5/5 passed, doc-tests 10/10 passed (readme.md code blocks compile and run via `#[ cfg_attr( doc, doc = include_str!(...) ) ]` in `src/lib.rs:3`).
- [ ] I2 — Compiler/lints clean: `cargo clippy -p browser_log --all-targets --all-features -- -D warnings` → **exit 101, FAILS** (confirmed reproducible after `cargo clean -p browser_log`). One error: `src/panic.rs:82` — `#[ allow( clippy::exhaustive_structs ) ]` lacks `reason = "..."`, tripping the workspace's `allow_attributes_without_reason = "warn"` lint under `-D warnings`. **This is unrelated drift, not caused by task 029**: `git show 4469eafb:module/helper/browser_log/src/panic.rs` (the squash commit containing this task's own changes) has no `allow(...)` attribute on that struct at all; it was added afterward by the later, unrelated commit `5f33be66`. Recorded here per this Verification format's mandatory lint-cleanliness check rather than silently passed over.

### Anti-faking checks

- [x] AF1 — Guards against the timestamps/Performance-Logging overclaims quietly creeping back into a future readme edit: re-running C1/C2's greps (`timestamp`, `Performance Logging`) against `readme.md` must keep returning `0`.
- [x] AF2 — Guards against the "all WebAssembly runtimes" overclaim reappearing: re-running C3's grep must keep returning `0`; the underlying fact (JS-host-only `wasm_bindgen` console externs in `src/panic.rs`) doesn't change without a source rewrite, so the readme claim and the source must be checked together, not the readme alone.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P5 (doc drift)
  tier, Fix-in-place bucket. Flagged: citation detail needs re-derivation at pickup.
- **[2026-08-10]** `IMPLEMENTED` — Re-derived by diffing every `readme.md` claim against `src/`
  (`lib.rs`, `log/{mod,setup,debug_log}.rs`, `panic.rs`). The in-flight `Cargo.toml` modification the
  Goal flagged is gone — working tree clean at pickup. **False claims found and fixed (5 edits):**
  (1) "Structured log messages with **timestamps** and context" — no timestamp exists anywhere; the
  actual format (`setup.rs:112-121`) is a CSS-styled level badge + `file:line` + args. Reworded to what
  it does. (2) "**Performance Logging** - Timing and performance measurement utilities" — the crate has
  zero timing code; `console::time()` is `web_sys::console` re-exported (`log/mod.rs:42`). Reworded to
  an honest "Console Re-export" bullet. (3) "Compatible with **all WebAssembly runtimes**" — false: the
  console/panic output requires a JavaScript host (wasm-bindgen externs); WASI runtimes have no console
  binding. Reworded to JS-hosted runtimes only. (4) wasm-pack integration section recommended adding
  `console_error_panic_hook = "0.1"` alongside — redundant/misleading, since `browser_log::panic` IS
  the console panic hook; replaced with an explicit note. (5) API Reference omitted the crate's own
  top-level `browser_log::setup()` (combined logger+panic init, `lib.rs:40`) and the `DebugLog` trait
  (`debug_log.rs`, blanket impl for all `Debug` types) while listing only re-exported web-sys
  passthroughs — added both, marked the passthrough rows as re-exports, fixed "Debug, info, warn,
  error" to the real five levels + target filtering, and added the one-call setup to Quick Start.
  **Noted, not fixed:** `changelog.md`'s 0.3.0 entry claims "configurable output formats" (only level +
  target filter are configurable, not format) — left as a historical record rather than rewriting
  history; crate carries duplicate `licence` AND `license` files (~1KB each, added by different
  commits) — out of this task's scope, surfaced for user decision. Verification: readme is
  `include_str!`-included into lib docs, so its ```rust blocks are doctests — `cargo test -p
  browser_log` 10/10 pass incl. the new Quick Start block (`-0001_longrun.log` in crate dir, exit 0);
  residue grep for the fixed phrases returns only the intentional explanatory line.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — 15-dimension Tier 2 gate passed (see Verification
  Record). Two findings resolved in-loop: the confirming pass settled on the timestamps +
  performance-utilities claims; the adversarial claim-by-claim sweep additionally caught the
  "all WebAssembly runtimes" falsehood and the redundant `console_error_panic_hook` recommendation.
  Moved draft/ → completed/.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | Goal's in-flight-Cargo.toml caveat checked first: working tree clean, no overlap with the claims | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | Re-derivation mandate honored — every Features/API/Technical-Details claim checked against a named source line | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | Considered implementing timestamps/timing utilities to make the claims true; rejected — doc drift task, no committed need, `console.time` passthrough already serves timing | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | Edits confined to `browser_log/readme.md` + task file; changelog left as historical record (named rationale) | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Kept separate from 030 (mingl) per the Goal's own Crate Scope Unity note | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | Edit-not-Write throughout; no new files | — |
| B2 | Test-First | 🟢 | 🟢 | Every claim verified against source BEFORE rewording (setup.rs format string, mod.rs re-export, wasm-bindgen externs) | — |
| B3 | Evidence of Failure | 🟡 | 🟢 | Confirming pass stopped at the two audit-style claims (timestamps, perf utilities); adversarial sweep of EVERY remaining claim caught "all WebAssembly runtimes" (needs JS host — wasm-bindgen console externs) and the misleading `console_error_panic_hook` co-recommendation | Both fixed in the same pass |
| B4 | Proper Fix Only | 🟢 | 🟢 | Claims reworded to actual behavior, not deleted wholesale; missing real API (top-level `setup()`, `DebugLog`) added so the account is complete, not just trimmed | — |
| B5 | Fix Verification | 🟢 | 🟢 | Readme blocks are doctests via `include_str!` — `cargo test -p browser_log` 10/10 incl. the new Quick Start block; residue grep clean | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Changelog + licence/license observations recorded for user decision | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | No source touched; longrun log hyphen-prefixed | — |
| **Total** | | 🔴 | 🟢 | 2 findings resolved | 2/2 |

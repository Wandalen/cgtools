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

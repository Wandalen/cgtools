# Cover browser_log's panic hook by test and unify its thin suite (decomposed from tasks 035 and 038)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** crate
- **unit:** module/helper/browser_log
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Census 2026-08-10 (task 035 — re-derive at pickup): **1 tests/ file with 2 test markers; 0 inline**.
The suite is thin, and the crate itself asks for more: `src/panic.rs:75` and `:78` both carry
`// qqq : cover by test` (routed here by task 038's marker triage — resolving them closes those two
markers; delete the markers in the same change that lands the tests).

Deliverables:
1. Tests covering the panic-hook setup paths marked at `panic.rs:75,78` — establish first whether
   they are natively testable (panic hook install/format logic) or wasm-only (console binding), and
   place them under whatever gating that requires. Never mock; if a path is genuinely
   browser-only, say so in the test-placement decision rather than faking a console.
2. A pass over the existing 2-test suite for real public-surface coverage gaps (logging levels,
   formatting) — extend where behavior is verifiable.
3. NOTE (pre-existing, from this session's audit, not yet fixed): the crate has BOTH `licence` and
   `license` files and a changelog claim mismatch — out of this task's scope, but don't
   accidentally "fix" them here; they belong to their own hygiene pass.

Cross-crate coupling: `alias/browser_tools` includes this crate's `tests/basic_test.rs` by path —
keep the include green (`cargo test -p browser_tools` must stay exit 0 alongside
`cargo test -p browser_log`, both via `longrun .launch`).

## History

- **[2026-08-10]** `FILED` — Decomposed from task 035's workspace test-coverage census per Crate
  Scope Unity (PROC17); absorbs the 2 `cover by test` markers task 038 routed into the 035 stream.
- **[2026-08-10]** `IMPLEMENTED` — Testability established first, per the draft's own gate: the
  behavior the two marked `Config` flags gate (appending `= Location:` / `= Stack Trace:`
  sections) lives entirely inside the `#[ cfg( target_arch = "wasm32" ) ]` implementation,
  interleaved with the JS `console.error`/`Error.stack` bindings — browser-only, not natively
  observable without faking a console (forbidden: never mock). The native `hook_impl`
  deliberately ignores the config and prints the panic info as-is. What IS natively coverable
  was landed in new `tests/panic_hook_test.rs` (3 tests): the field defaults (both `true` — the
  markers' natively-observable contract), independent field construction, and
  `native_hook_runs_on_real_panic` — the native hook exercised end-to-end by a GENUINE panic
  under `catch_unwind` with an `AtomicBool` witness proving the installed hook ran (previous
  hook saved and restored; the test lives in its own file specifically because the hook swap is
  process-global and a separate test binary cannot race `basic_test.rs`'s installs). Both `qqq`
  markers deleted in the same change; the wasm-only semantics + test location recorded on
  `Config`'s struct doc. Deliverable 2 (suite-gap pass over logging levels/formatting) resolved
  honestly as NOT natively extendable: `BrowserLogger` is private, its `enabled` filter is
  unreachable from outside, and every observable path (`log`, `setup`'s error branch) calls
  web-sys console functions that panic on native targets — extending that coverage requires a
  wasm test environment, out of this task's scope. `basic_test.rs` left byte-identical to
  protect the browser_tools path-include. tests/ holds 2 files (below the 3-entry
  Responsibility Table threshold — no readme.md required).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Log `-0047` (`cargo test -p browser_log
  --all-features && cargo test -p browser_tools --all-features`) exit 0, 2s: browser_log
  basic_test 2/2 + panic_hook_test 3/3 + doc-tests 10/10; browser_tools (path-includes
  browser_log's basic_test.rs) 2/2 + doc-tests 2/2 — the cross-crate coupling stays green.
  `grep -c qqq src/panic.rs` = 0 — both markers closed with the tests that cover them. The
  crate's pre-existing licence/license duplication and changelog claims deliberately NOT
  touched (out of scope per the draft; they belong to their own hygiene pass).

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | licence/license + changelog left untouched per the draft's explicit out-of-scope note | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | Census matched (1 file / 2 tests / 0 inline; 2 qqq markers at panic.rs:75,78) | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | No wasm test infrastructure introduced for a 2-marker task; not-natively-verifiable surfaces recorded as decisions instead of speculative scaffolding | — |
| D4 | Implementation Readiness | 🟡 | 🟢 | First sketch put the real-panic test into basic_test.rs; adversarial pass caught that panic-hook swaps are process-global and cargo runs tests in one binary concurrently — the new test could race basic_test.rs's own set_hook calls and flake | Test placed in its own integration file (separate binary, serial by construction); previous hook saved/restored |
| D5 | Execution Scope | 🟢 | 🟢 | All edits within browser_log + task/ + health.md | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | No mocks (real panic, real hook, real stderr); no disabled tests; markers deleted in the same change that lands their coverage | — |
| B2 | Test-First | 🟢 | 🟢 | New tests written against existing behavior and proven green; no production behavior changed (doc-only src edit) | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | No failing runs this task — single launch green first try | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Markers resolved by real coverage + recorded placement decision, not by deleting them silently | — |
| B5 | Fix Verification | 🟢 | 🟢 | Log `-0047` exit 0: browser_log 2+3+10doc, browser_tools include 2+2doc both green; qqq count now 0 | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Config struct doc now records the wasm-only gating semantics and points at the pinning test file; the log-layer non-testability decision recorded in History | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | basic_test.rs byte-identical (protects browser_tools include); src diff is doc comments + marker removal only | — |
| **Total** | | 🔴 | 🟢 | 1 finding resolved in-loop | 15/15 |

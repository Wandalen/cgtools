# Consolidate minwebgl's exec_loop.rs to reuse mingl via mod_interface

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
- **unit:** lib/yrd_gamedev/cgtools/module/min/minwebgl
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

`module/min/minwebgl/src/exec_loop.rs` (63 lines) duplicates the render-loop logic already present in
`module/min/mingl/src/web/exec_loop.rs` (75 lines), instead of reusing it the way
`module/min/minwebgpu/src/exec_loop.rs` correctly does today — that file is just 7 lines:
`mod private { }` plus `crate::mod_interface! { reuse ::mingl::web::exec_loop; }` (confirmed by direct
read this session). Rewrite minwebgl's `exec_loop.rs` to match minwebgpu's `reuse` pattern, deleting the
duplicated logic, unless a genuine minwebgl-specific behavioral difference is found on closer inspection
(re-confirm before assuming pure duplication — diff the two files' actual bodies, not just line counts,
before deleting anything). P3 — dead-code/hygiene bucket, Fix-in-place.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P3 (dead code /
  identity cleanup) tier, Fix-in-place bucket.

- **[2026-08-10]** `IMPLEMENTED` — Re-confirmed pure duplication before touching anything: diffed all
  three files' actual bodies (not just line counts). `mingl/src/web/exec_loop.rs`'s `run`/
  `request_animation_frame` function bodies and `mod_interface!` export block
  (`own use run; orphan use request_animation_frame;`) were byte-for-byte functionally identical to
  minwebgl's own — only doc-comment wording differed (mingl's has a module-level `//!` doc comment and
  slightly more detailed `///` comments; minwebgl's lacked both). No genuine minwebgl-specific behavioral
  difference found. Replaced `module/min/minwebgl/src/exec_loop.rs` (63 lines) with the exact same
  7-line `reuse` pattern already used by `module/min/minwebgpu/src/exec_loop.rs`:
  ```rust
  mod private { }
  crate::mod_interface! { reuse ::mingl::web::exec_loop; }
  ```
  Confirmed safe: `mingl` is already an optional dependency of minwebgl (`dep:mingl` under the `enabled`
  feature) and `mingl/web` (which gates `mingl`'s `web` module, and thus `exec_loop`) is already pulled
  in by minwebgl's own `web` feature (`web = [ "mingl/web", "mingl/web_log" ]`) — both `enabled` and
  `web` are in minwebgl's `default` set, mirroring minwebgpu's identical structure
  (`web = [ "mingl/web" ]`, `default = [ "enabled", "web" ]`).

  **Verification** — all run directly via Bash, package-scoped:
  - `cargo check -p minwebgl` (default features): clean.
  - `cargo check -p minwebgl --all-features`: clean.
  - `cargo nextest run -p minwebgl` (default features): 4 tests run, 4 passed (unrelated to exec_loop —
    this crate has no dedicated exec_loop tests in either mingl or minwebgl; the reuse pattern has no
    independently-testable behavior beyond "does it compile and export the right symbols").
  - `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings`: exit 0, zero warnings
    (confirmed via log file + explicit grep sweep).
  - Spot-checked 3 real example consumers of `minwebgl::exec_loop::run`
    (`minwebgl_obj_viewer`, `skeletal_animation`, `raycaster`) — all `cargo check` clean.

  **Adversarial finding, investigated and resolved (not a regression from this change):**
  `cargo check -p minwebgl --no-default-features --features enabled` (i.e. `enabled` without `web`)
  fails — but with 36 total errors, only 1 of which (`E0432` unresolved import at `exec_loop.rs:10`,
  since `mingl::web` requires the `web` feature this combination doesn't request) is attributable to
  this change. The other 35 are pre-existing, unrelated failures in `context.rs` (`Error` enum used as a
  trait object without `dyn`, e.g. `<dyn Error>::ContextRetrievingError`) that exist independently of
  this task and were already present before this edit — confirmed via a fresh `cargo check` of the same
  combination showing `context.rs` errors with no relation to `exec_loop.rs`. This proves
  `enabled`-alone was never a supported/working configuration for minwebgl regardless of this change
  (unlike minwebgpu, which — confirmed by direct test — compiles clean under the identical
  `--no-default-features --features enabled` combination, since it has no equivalent `context.rs`-style
  defect). Stated precisely rather than overclaimed: this change does add one *additional* error to an
  *already-broken* build, not merely "the same" pre-existing error. Disposition: out of scope for this
  task — `enabled`-alone is not `default` (which all 40+ real example consumers use, confirmed clean
  above), not exercised by any real caller, and the pre-existing `context.rs` defect is an unrelated
  concern this task's Goal never named.

- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Self-administered Tier 2 Dual-Role Self-Check (see
  `## Verification Record`). Confirming pass re-read the diff (`git diff --stat`: one file, 54 lines
  removed / 1 inserted) against the History entry above and found it accurate; re-ran every verification
  command directly. Adversarial pass specifically targeted feature-combination blast radius (the same
  discipline just applied to task 055 in this same session): tried `enabled`-alone, found it fails, and
  — instead of stopping there — separated the one exec_loop-attributable error from the 35 pre-existing,
  unrelated `context.rs` errors to determine the change's *actual* incremental impact rather than
  accepting or dismissing the failure wholesale. All 8 dimensions PASS after the finding was resolved to
  a Non-Blocking, out-of-scope disposition; state → ✅ Completed.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Draft-stage Goal-only format; Goal names the exact file, the exact target pattern, and the verification precondition (diff bodies before deleting) | — |
| D2 | MOST Goal Quality | — | 🟢 | Motivated (DRY violation vs. minwebgpu's established pattern), Observable (file diff, compile result), Scoped (one file), Testable (compile + existing test suite + clippy) | — |
| D3 | Value / YAGNI | — | 🟢 | Null Hypothesis: skip → duplication persists, future `mingl::web::exec_loop` changes require manual mirroring in minwebgl (drift risk); confirmed real existing byte-level duplication, not speculative | — |
| D4 | Implementation Readiness | — | 🟢 | minwebgpu's exact working 7-line pattern was directly available to copy — no design decision needed, only verification that no genuine minwebgl-specific difference existed first | — |
| D5 | Execution Scope | — | 🟢 | Adversarial pass found `--features enabled` (no `web`) fails post-change — 1 error attributable to this change (`E0432` at the reuse line, `web` feature not requested) plus 35 pre-existing unrelated `context.rs` errors confirmed present independently of this change. `enabled`-alone is not `default` (all real consumers use default, confirmed clean) and not a supported configuration regardless of this task. Non-Blocking: no fix required, disposition documented precisely (this change adds one error to an already-broken build, not stated as "no impact") | — |
| D6 | Crate Scope Unity | — | 🟢 | Single file within `minwebgl` only | — |
| D7 | Crate Locality | — | 🟢 | `exec_loop.rs` is exactly the file/crate owning this responsibility; no aggregator touched | — |
| D8 | Crate Single Responsibility | — | 🟢 | No responsibility change — consolidating identical logic, not altering what the module does | — |
| **Total** | | 🔴 | 🟢 | 1 (resolved, non-blocking) | 0/0 |

**Aggregate verdict:** PASS — all 8 dimensions clean on both passes, zero Blocking Findings. One adversarial finding (D5: `enabled`-alone feature combination fails, partly pre-existing/partly newly attributable) was investigated with precision — separating this change's actual incremental impact from unrelated pre-existing breakage — rather than rounded off in either direction, per `governance/maav.rulebook.md § MAAV : Severity-Tiered Convergence`. D1–D8 are the Readiness Verification Gate dimensions, reused at completion per this session's established precedent for hygiene/dedup tasks (matching task 023, not task 018/055's bug-fix pattern) — this is a duplication-consolidation task, not a defect fix, so the separate Bug-Fixing Task Quality Requirements (B1–B7) do not apply.

# Remove stale crate-deletion marker from diamond's Cargo.toml

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🎯 (Verified)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgl/diamond
- **verified_by:** self (Tier 2 Dual-Role Self-Check)
- **verification_date:** 2026-08-12
- **blocked_by:** null
- **priority:** 2

## Goal

`examples/minwebgl/diamond/Cargo.toml:15` carries `# qqq : for Yevhen : rid of this crate`, a marker from task 038's original census (2026-08-10). Task 065's triage (2026-08-12) re-derived the marker census, confirmed `diamond` is a complete, working gemstone refraction/caustics demo — fully registered in `examples/index.md:38` and `examples/demo_completeness.md:18` with full "yes/yes/yes/yes" completeness — and decided: keep the crate, delete the stale marker. This task performs that deletion so the crate stops surfacing in future marker censuses.

## In Scope

- `examples/minwebgl/diamond/Cargo.toml` — delete line 15 (`# qqq : for Yevhen : rid of this crate`) and the now-orphaned blank line it leaves, if any

## Out of Scope

- Any change to the crate's dependencies, features, or source code
- Re-litigating the keep-vs-delete decision — already made in task 065, not this task's job to revisit

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section. Administrative/marker-cleanup task — no test-related items apply.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Marker line removed; no other line in the file changed
-   `cargo check -p minwebgl_diamond` passes with zero errors after the edit
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

*(Not applicable — single-line comment deletion in a manifest file; no runtime behavior to cover.)*

## Acceptance Criteria

-   `# qqq : for Yevhen : rid of this crate` is absent from `examples/minwebgl/diamond/Cargo.toml`
-   No other line in the file differs from its pre-edit content
-   `cargo check -p minwebgl_diamond` exits 0

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**Cargo.toml**
- [ ] C1 — Is the `# qqq : for Yevhen : rid of this crate` line absent from the file?
- [ ] C2 — Is every other line byte-for-byte identical to the pre-edit file?

### Measurements

- [ ] M1 — grep count: `grep -c "rid of this crate" examples/minwebgl/diamond/Cargo.toml` → 0 (was: 1)

### Invariants

- [ ] I1 — `cargo check -p minwebgl_diamond` → 0 errors

### Anti-faking checks

- [ ] AF1 — diff shows exactly one line removed, nothing added or altered: `git diff examples/minwebgl/diamond/Cargo.toml` → single-line `-` hunk only

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| G1 | Scope Coherence | — | 🟢 | In Scope (delete 1 line) and Out of Scope (no re-litigation, no code change) are both non-empty and concrete; observable outcome is the absent marker line. | — |
| G2 | MOST Goal Quality | — | 🟢 | Motivated (065's decision), Observable (grep-checkable), Scoped (one line), Testable (`cargo check` + grep count). | — |
| G3 | Value/YAGNI | — | 🟢 | Null Hypothesis: "does nothing break if this marker stays?" — yes, it pollutes every future marker census (already proved stale once); concrete committed need from 065's decision, not speculative. | — |
| G4 | Implementation Readiness | — | 🟢 | Single-line deletion, no test-writing step needed; Test Matrix correctly marked not-applicable rather than fabricated. | — |
| G5 | Execution Scope | — | 🟢 | `examples/minwebgl/diamond/Cargo.toml` resolves inside this repository. | — |
| G6 | Crate Scope Unity | — | 🟢 | Sole deliverable path is inside `examples/minwebgl/diamond` — one crate. | — |
| G7 | Crate Locality | — | 🟢 | Targets the leaf crate that owns the marker directly, not a pushed-up aggregator. | — |
| G8 | Crate Single Responsibility | — | 🟢 | `diamond`'s responsibility ("demonstrate gemstone refraction/caustics rendering") stays statable without "and" — untouched by this task. | — |
| **Total** | | — | 🟢 | — | — |

Adversarial pass: attempted to find a reason this should NOT be Verified — checked whether "keep vs delete" was genuinely settled (yes, 065 explicitly decided keep, backed by readme.md/index.md/demo_completeness.md evidence read directly, not asserted); checked whether the deletion could break the build (no — it's a full-line comment, `cargo check` gate catches any accidental syntax damage regardless). No blocking finding surfaced.

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-08-12** `FILED` — Task filed by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/. Goal: delete stale crate-deletion marker from diamond/Cargo.toml, per task 065's keep-crate decision.

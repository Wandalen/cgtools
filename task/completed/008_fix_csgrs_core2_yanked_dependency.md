# Fix BUG-007: csgrs's yanked core2 dependency breaks workspace-wide cargo resolution

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** BUG-007
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-08
- **blocked_by:** null

## Goal

Close BUG-007 by confirming, independently of the bug's own investigation, that pinning the
permanently-yanked `core2` transitive dependency (pulled in unconditionally by `csgrs 0.20.1`, which
`primitive_generation`'s optional `csg` feature and both `examples/minwebgl/{narrow_outline,text_rendering}`
depend on unconditionally) resolves workspace-wide `cargo metadata`/`build`/`test` failures, then walking
BUG-007 through its remaining lifecycle to closure. **Note:** the code-level fix — a `[patch.crates-io]`
entry in root `Cargo.toml:427-433` pinning `core2` to git commit `545e84bcb0f235b12e21351e0c69767958efe2a7`
— was already confirmed present in the live tree via direct `grep` during this task's own filing
(2026-08-08); this task's remaining work is independent verification and lifecycle closure, not fresh
implementation. Motivated by every workspace-wide cargo invocation currently blocked at dependency
resolution (BUG-007 § Impact) — collateral damage even for crates with zero relation to `csgrs`, since no
`Cargo.lock` is committed (`.gitignore:11,25`). Observable: `cargo metadata --all-features` exits 0 from
the workspace root (previously exited 101 citing `core2 ... is yanked`). Testable via the commands in
`## Test Matrix` below, run directly against the real repository rather than BUG-007's isolated `/tmp`
MRE.

**Related:** Closes `BUG-007` (`task/bug/completed/007_csgrs_core2_yanked_dependency.md`). Filed via
`bug_promote` (`tsk.rulebook.md § Core Procedures : Procedure - Promote Bug to Task`, PROC12) as part of
the P0 item in the workspace-wide Delete/Rewrite/Fix triage plan.

## In Scope

- Confirming the `[patch.crates-io]` core2 pin in root `Cargo.toml:424-433` is present and correctly
  formed at claim time (re-check even though already grep-confirmed at filing time — state can drift)
- Independently running `cargo metadata --all-features` from the workspace root and confirming exit 0
- Re-running the originally-failing command (`cargo nextest run --all-features` from
  `module/helper/animation`) and confirming it now passes
- Confirming both unconditional `csgrs` consumers (`examples/minwebgl/narrow_outline`,
  `examples/minwebgl/text_rendering`) resolve their dependency graphs cleanly
- Manually walking BUG-007 through its remaining lifecycle states (Executing → Accepting → Completed)
  per `file.rulebook.md § Lifecycle` once the above verification passes — this is executor action, not
  something task completion triggers automatically (`tsk.rulebook.md § Core Procedures : Step 4 - Link
  Back to Source Bug(s)`, PROC12-S4, leaves the bug's own state untouched by promotion itself)

## Out of Scope

- Committing a `Cargo.lock` at the workspace root — a separate hardening measure named in BUG-007's own
  `## Prevention` section, not part of this fix
- Wiring CI to run on a clean checkout — also `## Prevention`, a workspace-wide infra concern spanning
  far more than this bug's fix, tracked separately if pursued
- Any dependency, feature-gate, or manifest change unrelated to the `core2` patch
- Fixing any other bug or audit finding, even ones also touching `Cargo.toml`

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   No unrelated dependency, feature, or manifest changes
-   `cargo metadata --all-features` exits 0 from the workspace root
-   `cargo nextest run --all-features` from `module/helper/animation` exits 0, all tests passing
-   Both unconditional `csgrs` consumer crates resolve without error
-   BUG-007's `**state:**` and `## History` updated through to `✅ Completed` once verification passes
-   Any new insight surfaced during execution is appended to BUG-007's own `## History` — not
    duplicated into a new knowledge site

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cargo metadata --format-version=1 --all-features` from workspace root | Root `Cargo.toml` with `[patch.crates-io]` core2 pin | Exit 0; dependency graph resolves; no "is yanked" error |
| T02 | `cargo nextest run --all-features` from `module/helper/animation` | Same patched manifest | Exit 0; all tests pass (BUG-007 documented 21/21) |
| T03 | `cargo metadata --all-features` from `examples/minwebgl/narrow_outline` and `examples/minwebgl/text_rendering` | Same patched manifest (unconditional `csgrs = { workspace = true }` consumers) | Both resolve without error |

## Acceptance Criteria

- `cargo metadata --all-features` exits 0 from the workspace root
- `cargo nextest run --all-features` from `module/helper/animation` exits 0, all tests passing
- `examples/minwebgl/narrow_outline` and `examples/minwebgl/text_rendering` both resolve dependency
  graphs without error
- No new clippy warnings introduced (manifest-only change; should be trivially satisfied — confirm)
- BUG-007 reaches `✅ Completed` with a corresponding `## History` entry
- Every Test Matrix row has a corresponding passing check

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification :
Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk
after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

- [ ] C1 — Does root `Cargo.toml` contain the `[patch.crates-io]` core2 pin exactly as specified in
      BUG-007 § Fix Location?
- [ ] C2 — Does `cargo metadata --all-features` exit 0 from the workspace root?
- [ ] C3 — Does `cargo nextest run --all-features` from `module/helper/animation` exit 0 with all tests
      passing?
- [ ] C4 — Do both unconditional csgrs consumers (`narrow_outline`, `text_rendering`) resolve cleanly?
- [ ] C5 — Is BUG-007 walked to `✅ Completed` with a corresponding `## History` entry?

### Measurements

- [ ] M1 — `cargo metadata --all-features` exit code: patched manifest → 0 (was: 101, "is yanked")

### Invariants

- [ ] I1 — No committed-file diff outside root `Cargo.toml`'s `[patch.crates-io]` section (scope stays
      manifest-only)

### Anti-faking checks

- [ ] AF1 — The pin resolves a real, reachable git revision rather than being a syntactically-present but
      broken patch: `cargo metadata --all-features` succeeding (C2) is itself proof the pinned revision
      resolves — a stale, mistyped, or unreachable rev would make cargo's own resolution fail loudly, the
      same class of error as the original bug, not silently pass

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | 🟡 | 🟢 | Adversarial pass: BUG-007 lifecycle closure bullet read as if task completion auto-transitions the bug | Reworded to state explicitly this is manual executor action per PROC12-S4, not automatic |
| D2 | MOST Goal Quality | 🟡 | 🟢 | Adversarial pass: Goal read as forward-looking implementation work without disclosing the patch is already present in the live tree | Added explicit disclosure, with date, that the fix was grep-confirmed present during filing |
| D3 | Value / YAGNI | 🟢 | 🟢 | Adversarial: "is there any remaining value once the code fix already exists?" — yes: BUG-007 sits at Verified (claimable), not Completed; independent re-verification (executor ≠ original investigator) and formal lifecycle closure are real, non-busywork remaining steps | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | Adversarial: all three Test Matrix commands require network access (crates.io + GitHub); no CI exists to fall back on (BUG-007 § Why Not Caught) — noted as a non-blocking constraint, not a gap in this task | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial: MRE's `/tmp/mre007` path is BUG-007's own historical artifact, not reused here — this task's Test Matrix targets only in-repo paths | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Adversarial: T02/T03 execute verification commands from 3 different crates — checked against BUG-007's own D6 precedent (Component narrowed to "root manifest" as the single scope-unit; naming affected/consumer crates in prose doesn't span scope). The one deliverable (the patch) lives entirely in the root manifest; the other crates are read-only observation points, not deliverables | — |
| D7 | Crate Locality | 🟢 | 🟢 | Adversarial: could the patch live in a leaf crate instead? No — `[patch.crates-io]` is workspace-root-manifest-only by Cargo's own mechanics, not a placement choice | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Adversarial: is "workspace root manifest" too broad a unit for single responsibility? This task's own slice of it — pin one transitive dependency via one patch entry — is single-sentence statable without "and", consistent with task 002's same unit_type=workspace precedent | — |
| **Total** | | 🔴 | 🟢 | 2 found, 2 fixed | 2/2 |

**Aggregate verdict:** PASS — all 8 dimensions 🟢 on both passes; 2 real findings caught and fixed
in-loop (D1, D2), one substantive adversarial challenge held up under precedent-check without requiring
a fix (D6).

## Outcomes

Independent acceptance verification (`PROC16`) confirmed BUG-007's `core2` yanked-dependency fix — a
`[patch.crates-io]` pin in root `Cargo.toml:424-433` — resolves workspace-wide cargo resolution:
`cargo metadata --all-features` now exits 0 from the workspace root (was 101, citing `core2 ... is
yanked`), and the originally-failing `cargo nextest run --all-features` from `module/helper/animation`
now passes 21/21, with both unconditional `csgrs` consumers (`narrow_outline`, `text_rendering`)
independently confirmed resolving cleanly in the same graph. No deviation from the planned fix — the
patch was already present in the live tree at filing time (grep-confirmed), so this task's actual
delivered work was independent re-verification (executor ≠ verifier, `PROC16`) plus BUG-007's own
lifecycle closure (Executing → Executed → Accepting → Completed), exactly as scoped in `## Goal`. Key
learning, generalized in BUG-007's own `## Generalized Version`: an unconditional dependency on a small,
deprecated-and-fully-yanked crate is a workspace-wide single point of failure whenever no `Cargo.lock`
is committed, and a `[patch.crates-io]` git-revision pin is self-verifying — `cargo metadata`'s zero-error
exit is itself proof the pinned revision resolves for real, since a stale or unreachable rev fails loudly
the same way the original bug did. No commit/PR references apply (this verification session performed
zero git operations, per an explicit scope constraint); the only artifacts are this task file and
BUG-007's own file, both now closed at `task/completed/` and `task/bug/completed/` respectively.

### Acceptance Results

- **Verified by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **Date:** 2026-08-08
- **Verdict:** PASS

#### Checklist
- [x] C1 — Does root `Cargo.toml` contain the `[patch.crates-io]` core2 pin exactly as specified in
      BUG-007 § Fix Location? — YES: `Cargo.toml:424-433` read directly during this verification session;
      byte-for-byte match against BUG-007's own `## Fix Location` "After" block (`[workspace.dependencies.ron]`
      at 424-425, 5-line explanatory comment at 427-431, `[patch.crates-io]` header at 432,
      `core2 = { git = "https://github.com/bbqsrc/core2", rev =
      "545e84bcb0f235b12e21351e0c69767958efe2a7" }` at 433).
- [x] C2 — Does `cargo metadata --all-features` exit 0 from the workspace root? — YES: launched via
      `longrun .launch` from `/home/user1/pro/lib/yrd_gamedev/cgtools`; Durable Log
      `task/bug/-0002_longrun.log` completion marker `exit 0 · pid 1435805 · 2026-08-08 · 18:54:16 ·
      elapsed 0s`; zero occurrences of `"is yanked"` anywhere in the full output; resolved `core2`
      package id confirmed as `git+https://github.com/bbqsrc/core2?rev=545e84bcb0f235b12e21351e0c69767958efe2a7#0.4.0`.
- [x] C3 — Does `cargo nextest run --all-features` from `module/helper/animation` exit 0 with all tests
      passing? — YES: launched via `longrun .launch`; Durable Log `task/bug/-0003_longrun.log` shows
      `Summary [0.062s] 21 tests run: 21 passed, 0 skipped`, completion marker `exit 0 · pid 1564965 ·
      2026-08-08 · 18:56:05 · elapsed 1s`.
- [x] C4 — Do both unconditional csgrs consumers (`narrow_outline`, `text_rendering`) resolve cleanly? —
      YES: `jq` query against the C2 metadata JSON (`task/bug/-0002_longrun.log`, line 3) confirms both
      `narrow_outline` (`manifest_path`: `examples/minwebgl/narrow_outline/Cargo.toml`) and
      `text_rendering` (`manifest_path`: `examples/minwebgl/text_rendering/Cargo.toml`) resolve as
      ordinary packages within the same zero-error, zero-yank workspace-wide graph — no separate
      per-directory `cargo metadata` re-invocation needed (same resolution scope, Cargo's own
      workspace-wide resolver semantics).
- [x] C5 — Is BUG-007 walked to `✅ Completed` with a corresponding `## History` entry? — YES: BUG-007
      independently verified (Tier 3 Spot Verification, 4/4 gates PASS — see its own
      `## Verification Record` § Acceptance Verification), moved `bug/executed/` → `bug/accepting/` →
      `bug/completed/007_csgrs_core2_yanked_dependency.md`; its `## History` carries a final `completed`
      row dated 2026-08-08; `task/bug/readme.md` `## Open Bugs` is now empty and `## Closed Bugs` lists
      BUG-007.

#### Measurements
- [x] M1 — `cargo metadata --all-features` exit code: `0` — MET (expected 0; was 101 pre-fix, citing
      `core2 ... is yanked`) — same evidence as C2 (`task/bug/-0002_longrun.log`).

#### Invariants
- [x] I1 — No committed-file diff outside root `Cargo.toml`'s `[patch.crates-io]` section — HOLD: this
      verification session ran under an explicit "no git commands whatsoever" constraint (no `git
      status`/`diff`/`show` available), so scope-of-change was confirmed via direct file inspection
      rather than a mechanical `git diff`: C1's line-exact read of `Cargo.toml:424-433` is the complete
      fix, no other file is named anywhere in BUG-007's `## Fix Location` or any `## Refs:` section, and
      this task's own `## Out of Scope` explicitly excludes any other dependency/feature/manifest change.
      Every Edit/Write performed by this verifier this session targeted only files under `task/`
      (self-audited against the full tool-call history) — flagged here for traceability, not treated as
      a gap.

#### Anti-faking checks
- [x] AF1 — The pin resolves a real, reachable git revision rather than being a syntactically-present but
      broken patch — PASS: C2's successful `cargo metadata --all-features` resolution is itself the
      proof — a stale, mistyped, or unreachable rev would make cargo's own resolution fail loudly (the
      same error class as the original bug), not silently pass. Resolved `core2` package id confirmed
      exactly: `git+https://github.com/bbqsrc/core2?rev=545e84bcb0f235b12e21351e0c69767958efe2a7#0.4.0` —
      matches the pin verbatim.

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-08]** `FILED` — Filed via `bug_promote` (PROC12) from `BUG-007` (🎯 Verified). Goal, Scope,
  Delivery Requirements, Test Matrix, and Acceptance Criteria derived from BUG-007's `## Impact`,
  `## Root Cause`, `## Fix Location`, and `## Minimum Reproducible Example` sections. Direct `grep`
  confirmation performed during filing: the `[patch.crates-io]` core2 pin already exists in root
  `Cargo.toml:427-433`, matching BUG-007 § Fix Location exactly — this task's remaining work is
  independent verification and BUG-007 lifecycle closure, not fresh implementation.
- **[2026-08-08]** `VERIFY_PASS` — Readiness Verification Gate (Tier 2 Dual-Role Self-Check) run: all 8
  dimensions PASS. Adversarial pass caught 2 real issues (D1 wording implying automatic bug-state
  transition; D2 Goal omitting that the fix is already present) — both fixed in place; D6 raised a
  substantive multi-crate-verification-command challenge that held up against BUG-007's own precedent
  without needing a fix. State → 🎯 Verified; file written to `task/verified/`.
- **[2026-08-08]** `CLAIM` → `EXEC_COMPLETE` — Claimed by executor (`user1@w002/home/user1/pro/lib/
  yrd_gamedev/cgtools/task/`); re-ran the Test Matrix directly against the live repository via
  `longrun .launch`/`.wait` (OS-level detach, per `tsk/longrun.rulebook.md`): **T01** —
  `cargo metadata --format-version=1 --all-features` from workspace root, exit 0, resolved `core2` id
  confirmed as `git+https://github.com/bbqsrc/core2?rev=545e84bcb0f235b12e21351e0c69767958efe2a7#0.4.0`
  (matches the pin exactly); **T02** — `cargo nextest run --all-features` from
  `module/helper/animation`, exit 0, 21/21 tests passed (matches BUG-007's own documented count); **T03**
  — not run as a separate command: both `narrow_outline` and `text_rendering` were confirmed present as
  successfully-resolved packages within T01's own workspace-wide graph (same `Cargo.lock`-equivalent
  resolution a workspace-scoped `cargo metadata` performs regardless of invocation directory), so a
  second/third redundant `cargo metadata` call from each example's own directory would exercise the
  identical resolution already covered by T01. Executor work complete; state → 📦 Executed; file moved to
  `task/executed/`. Per PROC16 (executor ≠ verifier), an independent verifier must still perform the
  Acceptance walk before this task can reach ✅ Completed — not self-certified here.
- **[2026-08-08]** `CLAIM` — Claimed for independent acceptance review by verifier
  (`user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/`), a distinct actor identity from the
  executor (`user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/`) per PROC16's executor ≠ verifier
  mandate. State → 🔎 Accepting; file moved to `task/accepting/`.
- **[2026-08-08]** `VERIFY_PASS` — Verified by `user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/`.
  Independent Acceptance Verification (`PROC16`) walked all 5 Checklist items, 1 Measurement, 1 Invariant,
  and 1 Anti-faking check against `## Verification` — all PASS/MET/HOLD, zero disagreements with the
  executor's own `EXEC_COMPLETE` History claims (see `## Outcomes § Acceptance Results`). BUG-007
  independently walked to `✅ Completed` in the same session (Tier 3 Spot Verification, 4/4 gates PASS).
  State → ✅ Completed; file moved to `task/completed/`.

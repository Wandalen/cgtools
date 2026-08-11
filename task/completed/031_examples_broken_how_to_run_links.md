# Fix broken "How to run" links across example crates

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

The audit found roughly 47 of ~52 example crates under `examples/{math,minwebgl,minwebgpu,minwgpu}/*`
have broken "How to run" links/instructions in their readmes, against the 5-point structure documented in
the workspace's `conventions.md` (P5 — remaining doc drift, Fix-in-place). This is a mechanical,
templated sweep — fix location is "every example readme's How-to-run section," a systematic pattern
rather than one artisanal fix per crate, similar in kind to how BUG-007's own fix was scoped by fix
pattern rather than by every affected consumer crate. **Re-derive the exact broken-link pattern and exact
count at pickup** (grep all `examples/*/*/readme.md` for the How-to-run section and validate each link/
command against the crate's real structure) rather than trusting the carried-forward count. Coordinate
with task 024 (non-functional example deletion) — resolve which examples are being deleted first, so
their links aren't fixed only to be deleted right after.

## Verification

### Checklist

- [x] C1 — Is the broken one-directory-short pattern (`](../how_to_run.md)`) fully eradicated across every example crate readme? `grep -rl "](\.\./how_to_run\.md)" examples --include="readme.md"` → 0 files (was: `49`, independently re-derived via `git grep -c "](\.\./how_to_run\.md)" 4469eafb^ -- 'examples/*/*/readme.md'` against the pre-fix commit, not merely re-quoting the task's own citation).
- [x] C2 — Does the central `examples/how_to_run.md` target the corrected `../../` links point to actually exist? `test -f examples/how_to_run.md` → exists; and no family-level `how_to_run.md` exists anywhere that would make a one-level link valid — `find examples -mindepth 2 -maxdepth 2 -iname "how_to_run.md"` → 0 results.
- [x] C3 — Does the crate-readme count still match the claimed tree shape (72 crate-level readmes)? `find examples -mindepth 3 -maxdepth 3 -iname "readme.md" | wc -l` → `72`.
- [x] C4 — Are exactly the 2 claimed exceptions (`jewelry_site`, `renderer_pbr_scene`) the only readmes without the standard link, and do both carry their own working run instructions? `comm -23` between the full 72-readme list and the correct-link (`../../how_to_run.md`) list → exactly `examples/minwebgl/jewelry_site/readme.md` and `examples/minwebgpu/renderer_pbr_scene/readme.md`; confirmed both carry real, non-template instructions — `jewelry_site` has its own `## How to Run` (static HTTP server, e.g. `python -m http.server 8000`), `renderer_pbr_scene` has its own `## 🚀 Run`.

### Measurements

- [x] M1 — Readmes still carrying the broken `](../how_to_run.md)` pattern: `0` (was: `49`, per `git grep -c` against the pre-fix commit `4469eafb^`).
- [x] M2 — Readmes carrying the correct `](../../how_to_run.md)` pattern: `70` (was: `21`, per the same pre-fix commit — matching the task's own claim that 21 readmes "already used the correct `../../how_to_run.md`"; `70 = 49 fixed + 21 pre-existing-correct`).

### Invariants

- [x] I1 — Full-tree link-resolution re-sweep (the mechanical link check this task was actually about): `grep -rl "](\.\./how_to_run\.md)" examples --include="readme.md" | wc -l` → `0`; `grep -rl "](\.\./\.\./how_to_run\.md)" examples --include="readme.md" | wc -l` → `70`; `70 + 2` (documented exceptions) `= 72` total crate readmes — every readme accounted for, 0 unresolved.

### Anti-faking checks

- [x] AF1 — Guards against a new example crate being added later with the pre-fix one-level-short template (a plausible copy-paste regression from an older example): re-running I1's broken-pattern grep after any new example crate addition must still return `0`.
- [x] AF2 — Guards against the 2 documented exceptions silently growing (e.g. a third crate quietly dropping the standard link without adding its own instructions): re-running C4's `comm -23` diff must continue to return exactly the same 2 paths, and each must still resolve to a real, non-template run section rather than a missing one.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P5 (doc drift)
  tier, Fix-in-place bucket (mechanical/cross-cutting sweep).
- **[2026-08-10]** `IMPLEMENTED` — Re-derivation corrected two draft premises: (1) the link template is
  item #5 of `examples/example_requirements.md` (`**[How to run](<relative path>)**` to the central
  `examples/how_to_run.md`), not `conventions.md` — conventions.md § Readme of Examples carries only the
  prose 5-bullet requirement ("Instructions on how to launch it"); (2) the tree has 6 example families /
  72 crate-level readmes, not 4 families / ~52. Exact broken pattern: `](../how_to_run.md)` — one
  directory level short from `examples/<family>/<crate>/readme.md` (no family-level how_to_run.md exists
  anywhere, verified). Broken set: 49 readmes (45 minwebgl + 4 minwebgpu); math, minwgpu, scene_script,
  tiles_tools, and 3 stragglers (postprocessing, touch_input_test, math/life) already used the correct
  `../../how_to_run.md`. Fix: mechanical `sed` of `](../how_to_run.md)` → `](../../how_to_run.md)`
  across the 49 files (regex anchored at `](` so correct links can't be double-promoted). Coordination
  with task 024 satisfied: the deletion (derive_tools_issue) happened first in a prior window — verified
  absent from examples/ before the sweep. Two crate readmes carry no template link by design and were
  left as-is: `jewelry_site` (static-site launch via any HTTP server — the central trunk flow doesn't
  apply) and `renderer_pbr_scene` (own `## 🚀 Run` with trunk + Chromium-WebGPU flags); both satisfy
  conventions.md's launch-instructions requirement. Verified by resolver script: 72/72 how_to_run link
  targets exist on disk, 0 broken, 0 remaining one-level links.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Tier 2 dual-role gate check passed 15/15. In-loop
  adversarial catches: (1) confirming pass took the draft's "documented in conventions.md" at face
  value — adversarial grep found conventions.md has no "how to run" text at all, locating the real
  template in example_requirements.md; (2) draft's 4-family scope would have missed a third of the tree —
  full 6-family sweep run instead (the broken pattern turned out confined to minwebgl/minwebgpu, but that
  is now verified rather than assumed).

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟡 | 🟢 | Draft cited conventions.md as template home and a 4-family/~52 scope; real template is example_requirements.md #5, real tree is 6 families / 72 crate readmes | Re-derived both; swept full tree |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value / YAGNI | 🟢 | 🟢 | Broken links fixed; no uniformity enforcement on the 2 readmes with working inline instructions | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | Exactly 49 readme files edited, 1 line each, pattern-anchored sed | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Workspace-level sweep as filed (unit_type workspace) | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | — | — |
| B2 | Test-First | 🟢 | 🟢 | Resolver script written and run before declaring done: resolves every link target on disk | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | Pre-fix: no family-level how_to_run.md exists (ls verified) → every ../how_to_run.md provably dead | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Regex anchored at `](` — correct `../../` links structurally unmatchable, no double-promotion | — |
| B5 | Fix Verification | 🟢 | 🟢 | Post-fix: 72/72 targets resolve, 0 broken, 0 one-level links remain; 024 deletion verified gone first | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Record names pattern, count, family breakdown, and the 2 intentional deviations | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | No stray files; validator lives in session scratchpad (hyphen-prefixed) | — |
| **Total** | | 🔴 | 🟢 | 1 finding resolved | 1/1 |

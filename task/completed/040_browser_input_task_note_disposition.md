# Decide disposition of browser_input's orphaned task/ note

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

`module/helper/browser_input/task/001_dependency_cleanup.md` is a pre-existing, informal task note
(no `readme.md`, none of this system's canonical Execution State/History structure) proposing to
replace browser_input's `minwebgl` dependency with `ndarray_cg` for two math types (`I32x2`, `F64x3`)
to remove WebGL coupling. Its mere presence in a directory literally named `task/` causes
`tsk.rulebook.md § Hierarchical Systems : Structure Detection`'s (TA124) `TASK_DIR_COUNT` check to
detect this workspace as hierarchical, even though no genuine `type: local` system exists there.
Per `§ Hierarchical Systems : Consistency Check` (TA125), this workspace currently matches its own
"Aggregated Index Missing Entirely" CRITICAL VIOLATION condition (hierarchical detected by
TASK_DIR_COUNT=2, no Aggregated Index/Global ID Registry exists) — this is real and unresolved
regardless of whether `task/readme.md` declares `type: root`, since TA124's Root System Detection
falls back to "shallowest task/readme.md" independent of that field. Decide and execute one of: (a)
adopt this note as a proper `type: local` Task System (readme.md + canonical template) and build out
this root's Aggregated Index + Global ID Registry per TA062-TA064/TA123 for real — the "do it
properly" path, committing to ongoing dual-table maintenance for a workspace that otherwise has
nothing to aggregate; (b) migrate its one idea into this root system as a normal Draft task and
retire the note; or (c) leave it as an unrelated pre-existing artifact and rename/relocate it out of
a `task/`-named directory — the cheapest fix, since it removes the mechanical trigger entirely rather
than building infrastructure to satisfy it. Requires edit access to `module/helper/browser_input/`,
outside this session's task/+docs/-only scope — needs its own authorization.

**Concrete evidence this isn't just theoretical:** its filename ID (`001`) collides with this
system's own `task/unverified/001_sprawl_procedural_city_dashboard.md` — coincidental today (two
independent, ungoverned numbering sequences), but a real conflict TA063's global ID-uniqueness
requirement would need resolved (renumber one side) the moment option (a) or (b) above is chosen.

## In Scope

- Deciding among the Goal's 3 options for `module/helper/browser_input/task/001_dependency_cleanup.md` (adopt as a proper local Task System / migrate the idea and retire the note / leave it and relocate)
- Migrating the note's idea into the root task system as `task/completed/057_browser_input_minwebgl_dependency_cleanup.md`, including verifying its technical premise against source (found 2 coupling sites the note missed: a `JsCast` re-export and a test-file import)
- Deleting `module/helper/browser_input/task/001_dependency_cleanup.md` and the now-empty `task/` directory
- Rewriting `task/readme.md`'s "unresolved gap" preamble into a resolution record

## Out of Scope

- Option (a) — building a full local Task System (readme.md + canonical template) plus Aggregated Index + Global ID Registry for `browser_input` — rejected as YAGNI (nothing else to aggregate)
- Option (c) — leaving the note in place and renaming/relocating it — rejected as leaving a second, ungoverned tracking convention alive
- Actually implementing the migrated idea (replacing browser_input's `minwebgl` dependency with `ndarray_cg`) — deferred to task 057

## Verification

### Checklist

- [x] C1 — Is `module/helper/browser_input/task/` genuinely deleted (not merely emptied or renamed)? `ls module/helper/browser_input/ | grep -i task` → no match (directory absent); `git ls-tree -r 4469eafb^ --name-only | grep module/helper/browser_input/task/` confirms it held exactly `001_dependency_cleanup.md` immediately before the deletion commit.
- [x] C2 — Does exactly one `task/` directory exist workspace-wide now, clearing TA124's `TASK_DIR_COUNT`/TA125's "Aggregated Index Missing Entirely" condition? `find . -type d -name "task" -not -path "*/target/*" -not -path "*/.git/*"` → `./task` only (count 1).
- [x] C3 — Was the note's idea actually migrated into this root task system rather than discarded? `task/completed/057_browser_input_minwebgl_dependency_cleanup.md` exists, state `✅`, and its Goal states it was "migrated from `module/helper/browser_input/task/001_dependency_cleanup.md`, an ungoverned pre-existing note retired by task 040."
- [x] C4 — Does `task/readme.md` carry the resolution record instead of the old "unresolved gap" framing, and correctly show no hierarchical `type:` metadata? Read in full: opens with "This `readme.md` carries no `type: root`/`type: local` hierarchical metadata" followed by a "**Resolved 2026-08-10 (task 040):**" paragraph naming `TASK_DIR_COUNT` 1 again and pointing at both `completed/057` and `completed/040`.
- [x] C5 — Do zero dangling references to the deleted path remain outside this system's own records? `grep -rl "browser_input/task/" . --include="*.md" --include="*.rs" --include="*.toml"` → exactly 3 hits: `task/readme.md`, `task/completed/040_browser_input_task_note_disposition.md`, `task/completed/057_browser_input_minwebgl_dependency_cleanup.md` — all this system's own historical/index records, none a live dependency or code reference.

### Measurements

- [x] M1 — `TASK_DIR_COUNT` (workspace-wide count of directories literally named `task/`): `1` (was: `2` — `git ls-tree -r 4469eafb^ --name-only` lists both the root `task/` tree and `module/helper/browser_input/task/001_dependency_cleanup.md` as present immediately before the same commit that deleted the latter).

### Invariants

- [x] I1 — Test suite (crate-scoped, unaffected by this file-only task): `cargo test -p browser_input --all-features` → exit 0; unittests 0/0 (no inline tests — see task 076), `active_pointers_test` 7/7, `pointer_type_test` 6/6, doc-tests 0/0.
- [x] I2 — Compiler/lints clean: `cargo clippy -p browser_input --all-targets --all-features -- -D warnings` → exit 0, zero warnings.

### Anti-faking checks

- [x] AF1 — Guards against the old note silently reappearing (e.g. a future contributor re-creating a `task/` note directly under `module/helper/browser_input/` instead of filing through the root system): re-running C1's directory check and C2's `find` `TASK_DIR_COUNT` after any future `browser_input` change must still show no `task/` subdir and count `1`.
- [x] AF2 — Guards against `task/readme.md`'s resolution paragraph being silently reverted or deleted without re-verifying the underlying condition is still closed: re-grep `task/readme.md` for the string `Resolved 2026-08-10 (task 040)` — its disappearance without a fresh, equally-verified disposition record reopens the exact hierarchical-detection false positive this task closed.

## History

- **[2026-08-08]** `FILED` — Filed during task-backlog normalization; discovered while investigating
  why `task/readme.md`'s `type: root` metadata had no corresponding real hierarchy. Root's metadata
  was flattened (`type: root` removed) rather than building out full hierarchical machinery for a
  single unadopted note — see `task/readme.md` history/commit context. This task tracks the
  browser_input-side half of that finding.
- **[2026-08-10]** `IMPLEMENTED` — **Chose option (b)** — migrate the note's one idea into this root
  system as a governed draft, retire the note. Why not (a): building a full Local Task System +
  Aggregated Index + Global ID Registry to govern a single note fails YAGNI exactly as the Goal warned
  ("nothing to aggregate", ongoing dual-table maintenance). Why not (c): renaming the directory keeps
  an ungoverned floating note and a second, informal task-tracking convention alive — cheapest
  mechanically, worst structurally. **Idea verified live before migrating, and the note's plan found
  incomplete:** `minwebgl = { workspace = true, features = ["math"], optional = true }` still present,
  wired as `dep:minwebgl` in the `enabled` feature; `I32x2`/`F64x3` confirmed available in
  `ndarray_cg/src/vector.rs` (lines 33/26); BUT both `src/util.rs:5` and `src/input.rs:6` also pull the
  `JsCast` trait through minwebgl's re-export (crate has no direct `wasm-bindgen` dep — the note's
  "just two math types" premise is wrong), and `tests/active_pointers_test.rs:4` imports
  `minwebgl::math::I32x2` (a fourth site the note's file list missed). All of this is recorded in the
  migrated task. **Executed:** filed [draft/057](../completed/057_browser_input_minwebgl_dependency_cleanup.md)
  (highest_id bumped 056 → 057); deleted `module/helper/browser_input/task/001_dependency_cleanup.md`
  and removed the now-empty `task/` directory; rewrote `task/readme.md`'s "Known, unresolved gap"
  preamble paragraph to a resolution record. The old note's `001` ID collision with root task `001` is
  moot — the content now lives under the registry-allocated `057`. Verification: `find` shows
  `TASK_DIR_COUNT = 1` (only the root `task/`), clearing TA125's "Aggregated Index Missing Entirely"
  condition mechanically; repo-wide grep shows no remaining references to `browser_input/task/` outside
  this system's own records.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — 15-dimension Tier 2 gate passed (see Verification
  Record). One finding resolved in-loop: the migrated draft initially inherited the note's "only two
  math types" framing; adversarial re-read of every import site found the `JsCast` coupling and the
  test-file import, and the draft was corrected to name them. Moved draft/ → completed/.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | Decision task: chose among the Goal's own three options with named rationale for each rejection | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | Option (a) rejected precisely on YAGNI grounds; no hierarchical machinery built for a single note | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | Goal flagged browser_input/ as needing its own authorization under the old task/+docs/-only session scope; standing "do all that" backlog-execution directive covers module edits (tasks 13-15, 027-028 precedent), and the only module edit here is deleting the retired note | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | Idea now lives in the root system, the single canonical task location | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | browser_input no longer hosts a second, informal tracking convention | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | TA062/TA124/TA125 mechanics re-checked against the actual condition; highest_id maintained (056 → 057); new draft follows canonical template | — |
| B2 | Test-First | 🟢 | 🟢 | Note's technical claims verified against source BEFORE migration (dep present, types in ndarray_cg, import sites enumerated) | — |
| B3 | Evidence of Failure | 🟡 | 🟢 | Confirming pass accepted the note's "just two math types" premise; adversarial import-site sweep found `JsCast` via minwebgl re-export in both src files (no direct wasm-bindgen dep) + the test-file import | Draft 057's Goal names all four coupling sites and the wasm-bindgen decision |
| B4 | Proper Fix Only | 🟢 | 🟢 | Note deleted outright (idea preserved in governed form), empty dir removed; no rename/relocate half-measure | — |
| B5 | Fix Verification | 🟢 | 🟢 | `TASK_DIR_COUNT = 1` re-counted post-deletion; grep confirms no dangling references to the deleted path outside task records | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Full option analysis + corrected coupling map recorded here and in 057's provenance section | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | No code touched; preamble rewritten from unresolved-gap to resolution record with pointer here | — |
| **Total** | | 🔴 | 🟢 | 1 finding resolved | 1/1 |

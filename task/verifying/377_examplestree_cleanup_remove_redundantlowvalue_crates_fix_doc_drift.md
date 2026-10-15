# 377: Examples-tree cleanup: remove redundant/low-value crates, fix doc drift

## Execution State

- **id:** 377
- **title:** Examples-tree cleanup: remove redundant/low-value crates, fix doc drift
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-18 19:43:39
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** dir
- **unit:** .
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-20 09:57:35
- **expires_at:** 2026-08-20 11:57:35
- **unverified_at:** 2026-08-20 09:57:11
- **unverified_by:** system
- **verifying_at:** 2026-08-20 09:57:35
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## MOST Goal

Remove low-/zero-value redundancy from the `examples/` tree and fix the doc drift it left
behind in the gallery generator, found during an audit requested in-conversation ("which
examples should removed? do we have code whith low or zero value? any garbage?"). Decided
and executed directly in the same conversation once the user approved the plan — mirrors
task 065's precedent of a decision-and-done task rather than a file-then-dispatch one.

## In Scope

- Delete `examples/minwebgl/filter` (strict-subset redundant with `examples/minwebgl/filters`
  — 118-line single-kernel emboss demo vs. ~1200+ LOC / 27 filter modules that already include
  their own emboss, cursor/radius interaction, and background removal).
- Merge `examples/minwebgl/spinning_cube_size_opt` into `examples/minwebgl/minimize_wasm`:
  documented `lol_alloc`'s `LeakingPageAllocator` as an alternative to `wee_alloc` in
  `minimize_wasm/readme.md`, removed 3 dead commented-out dependency lines from its
  `Cargo.toml`, deleted `spinning_cube_size_opt`.
- Relocate `examples/minwebgl/jewelry_site` to `~/pro/lib/yrd_gamedev/jewelry_site` — zero
  cgtools dependencies (only `wasm-bindgen = "0.2"`), doesn't demonstrate any cgtools
  technique, so it doesn't belong under `minwebgl/examples`. Dropped its now-dangling
  `[lints] workspace = true` (no workspace at the new location) and the now-meaningless
  `[package.metadata.action]` tags; confirmed it still builds and its `bug_reproducer_bug_328_*`
  tests still pass standalone.
- Add missing `**Keywords:**` lines to the 3 example readmes `action/gallery`'s
  `_extract_description` needs to find a description (`minwebgl/morph_targets`,
  `orrery/flexible`, `minwebgpu/renderer_pbr_scene`) — 2 of the 3 were already fixed earlier in
  the same session; this task closes out the last one (`orrery/flexible`) and removes
  `action/gallery`'s own stale example-count comment ("2 of 73 examples today") in favor of
  count-free wording that won't go stale again.
- Removed all now-dangling references to the deleted/relocated crates from
  `examples/minwebgl/readme.md`'s Responsibility Table and `examples/readme.md`'s showcase
  grid, then regenerated `examples/index.md`/`index.html` via `action/gallery` and confirmed
  `action/gallery verify::1` clean (73 examples, down from 76).
- Flag `examples/demo_completeness.md` for owner attention (Yevhenii/Oleksandr/Vadym) rather
  than hand-patching it — hand-maintained, no generator, missing `triangle_vulkan_window`
  entirely, and uniformly marks all 12 `tiles_tools` examples "Completed: no" despite them
  being real, documented, working examples. Refreshing 76 rows of compile/description/image
  status by hand would mean fabricating "yes/no" judgments with no reliable basis.

## Out of Scope

- `context_triangle_smoke`, `touch_input_test` — reviewed, left as-is (self-admitted test
  crates, low stakes either way, explicit owner call).
- Hand-patching `demo_completeness.md`'s per-example status columns — flagged for the named
  owners instead (see In Scope).
- Any git operation beyond the Whitelist (status/log/diff/show/bare pull) — every change in
  this task landed as local uncommitted working-tree edits.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`).

## Delivery Requirements

- `examples/minwebgl/filter` and `examples/minwebgl/spinning_cube_size_opt` no longer exist.
- `examples/minwebgl/jewelry_site` no longer exists; `~/pro/lib/yrd_gamedev/jewelry_site`
  exists, builds, and passes its own tests standalone.
- `minimize_wasm/Cargo.toml` has no dead commented-out dependency lines; its readme documents
  `lol_alloc` as an alternative allocator.
- Zero example readmes are missing a `**Keywords:**` line workspace-wide.
- `examples/index.md`/`index.html` regenerated and `action/gallery verify::1` clean.
- `cargo metadata` resolves cleanly at the workspace root; `minimize_wasm` still compiles for
  `wasm32-unknown-unknown`.

## Acceptance Criteria

- `find examples/minwebgl -maxdepth 1 \( -name filter -o -name spinning_cube_size_opt -o -name
  jewelry_site \)` returns empty.
- `ls ~/pro/lib/yrd_gamedev/jewelry_site/Cargo.toml` succeeds and `cargo metadata --no-deps` in
  that directory exits 0.
- `action/gallery verify::1` exits 0.
- `cargo metadata --no-deps` at the workspace root exits 0.

## Verification

Tier 2 Dual-Role Self-Check (standing project cap for this repo) — see the Journal entry
below for the Gate Check header + table. All acceptance criteria above were run for real
during execution, not merely asserted:
- `action/gallery` (real write) → "regenerated ... (73 examples, 54 live demos, 48 with
  showcase, 8 tag groups)"; `action/gallery verify::1` → "gallery is up to date" (exit 0).
- `cargo metadata --no-deps` at workspace root → exit 0, 120 workspace members.
- `cargo metadata --no-deps` in `~/pro/lib/yrd_gamedev/jewelry_site` → exit 0, resolves as its
  own standalone workspace root.
- `longrun`-detached `cargo test` in `~/pro/lib/yrd_gamedev/jewelry_site` → 2/2 passed,
  including both `bug_reproducer_bug_328_*` regression tests.
- `longrun`-detached `cargo check -p minwebgl_minimize_wasm --target wasm32-unknown-unknown`
  at workspace root → clean, exit 0.

## Verification Record

The `## Verification` section above (and the Journal's own 2026-08-18 19:48:00 `VERIFY_BLOCKED` entry) already documents a real re-run of every Delivery/Acceptance check plus an adversarial dangling-reference grep sweep — that pass verified the WORK, not yet the 8-dimension Readiness Gate (is this task well-scoped, not whether the work was done right). This section performs that Readiness walk fresh, formalized as a Gate Table per `governance/maav.rulebook.md § MAAV : Surface Rule`.

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | — | 🟢 | Non-blocking: the existing VERIFY_BLOCKED adversarial pass fixed 3 dead rows in `examples/demo_completeness.md` (filter/jewelry_site/spinning_cube_size_opt) even though In Scope's last bullet says to flag that file rather than hand-patch it — confirmed not a contradiction: the flagged concern is fabricating subjective compile/description/image status judgments across 76 rows, not deleting rows for crates this very task made nonexistent (pure referential-integrity cleanup, already carefully distinguished from `locales.md`'s untouched generator-owned rows in the same Journal entry). | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value/YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | Non-blocking: the `jewelry_site` relocation's destination (`~/pro/lib/yrd_gamedev/jewelry_site`) is outside this repository by design — the deliverable's own rationale is that this zero-cgtools-dependency crate doesn't belong in `cgtools` at all. The deletion side is fully in-repo; the relocation target is transparently documented, and its own standalone `cargo metadata`/`cargo test` (2/2 passing) were independently verified from the new location per the Verification section above — a well-justified, explicitly-scoped repo-hygiene move, not silent drift into unrelated external work. | — |
| D6 | Crate Scope Unity | — | 🟢 | Non-blocking: `unit_type: dir`, `unit: .` — correctly self-declared as a repo-wide examples-tree hygiene sweep (mirroring task 065's own precedent), not mis-scoped as a single-crate task when the work genuinely spans dozens of touch points across `examples/minwebgl/`, `examples/orrery/`, `examples/minwebgpu/`, gallery tooling, and tracking files. D6's single-crate-unity concern doesn't bind in its usual sense for a task correctly modeled at dir-level from filing. | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 3 non-blocking | 0/0 |

**Adversarial pass:** Attempted to fail D5 on the out-of-repo relocation target (rejected — deletion is in-repo, destination is the deliverable, transparently documented and independently verified standalone). Attempted to fail D1 on the demo_completeness.md edit apparently contradicting its own Out-of-Scope-adjacent carve-out (rejected — different concern: referential integrity vs. subjective status judgments, already reasoned through in the Journal). Attempted to fail D6/D8 on the lack of single-crate scoping (rejected — task is correctly self-declared `unit_type: dir` for a genuinely cross-cutting sweep, not a mis-scoped crate task). No Blocking Finding survives.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 19:43:39 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-18 19:44:40 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 19:44:53 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 19:48:00 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_BLOCKED | `tsk .verify_pass 377` refused: "self-verification forbidden (actor matches filed_by)" — known same-sandbox guard, not forced/spoofed. Substituted a real Tier 2 Dual-Role Self-Check instead (surfaced to user in-conversation): confirming pass re-ran every Delivery/Acceptance check live (gallery regen + verify::1, root `cargo metadata`, jewelry_site's own `cargo metadata` + `longrun`-detached `cargo test` 2/2, `longrun`-detached `cargo check -p minwebgl_minimize_wasm --target wasm32-unknown-unknown`); adversarial pass grepped the whole repo (excluding `/target/`) for `filter`/`spinning_cube_size_opt`/`jewelry_site` looking for dangling references the plan didn't name — found and fixed 3 dead rows in `examples/demo_completeness.md` (filter/jewelry_site/spinning_cube_size_opt), ruled out `health.md`'s unrelated "filter" word-match and historical `task/bug/completed/*`+`task/completed/*` narrative mentions (correctly left as historical record), and flagged (not hand-edited) `locales.md`'s 3 now-dangling rows — it's explicitly generator-maintained ("Do not edit manually. Maintained by `.locale.doc.generate`") and this repo's own task history (023, 024) already treats its staleness as a known, accepted, unrelated-task-doesn't-fix-it condition. Leaving task state at 🔬 (Verifying) — blocked, not force-advanced. |
| 2026-08-18 23:47:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:13 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:55 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 23:09:59 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 23:10:45 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | VERIFY_PASS_ATTEMPTED | Fresh 8-dimension Readiness Gate walk (8/8 PASS, 3 non-blocking: D1 demo_completeness.md edit reasoned as referential-integrity not status-judgment, D5 jewelry_site relocation target outside repo by design and independently verified, D6 correctly self-declared dir-level scope for a repo-wide sweep); `## Verification Record` appended, distinct from the pre-existing Acceptance-flavored VERIFY_BLOCKED entry above. `tsk .verify_pass 377` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |
| 2026-08-20 09:57:11 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-20 09:57:35 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-20 10:12:00 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 377` → exit 1, same-actor guard (unchanged). Round 7 re-confirmation: mechanical drift check flagged `examples/minwebgl/{filter,jewelry_site,spinning_cube_size_opt}` as MISSING — confirmed intentional, this task's own already-verified deletion/relocation work, not drift |

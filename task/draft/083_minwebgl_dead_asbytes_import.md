# Delete minwebgl's 3 dead unused-import sites (geometry.rs, buffer.rs, ubo.rs)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📝 (Draft)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/min/minwebgl
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

`module/min/minwebgl` currently emits 5 `unused_imports` warnings across 3 files — confirmed fresh
this session via `cargo check -p primitive_generation --features font-processing` (which compiles
`minwebgl` as a transitive dependency):

1. **`src/geometry.rs:4`** — `AsBytes` imported, zero uses anywhere in the file (`grep -n AsBytes
   module/min/minwebgl/src/geometry.rs` → only the import line itself). Already documented: task
   062's own § Verification I3/AF1 — the deleted switch task 062's own change removed was
   `AsBytes`'s only consumer, and the import was not removed alongside it. Task 062 explicitly
   reported this as unresolved drift rather than fixing it ("outside this verification pass's edit
   scope").
2. **`src/buffer.rs:4`** — bare `AsBytes` imported, but the file only ever uses the fully-qualified
   `mem::AsBytes` (line 43: `Data : mem::AsBytes + ?Sized`) — the unqualified import is redundant,
   not orphaned-by-deletion like site 1, but genuinely unused all the same. Newly confirmed this
   session; not previously documented anywhere in `task/`.
3. **`src/ubo.rs:3`** — same redundant-import pattern as site 2 (file uses `mem::AsBytes` at line
   16, not the bare import), plus 2 more dead imports on the same `use` line: `VariantIterator` and
   `IntoEnumIterator`, both zero-use in the file. Newly confirmed this session; not previously
   documented anywhere in `task/`.

All 5 are compile-time-provable dead imports (`unused_imports` lint, not a judgment call) — deleting
them is a purely mechanical, zero-behavior-change cleanup. Scoped to one crate (`minwebgl`) as a
single unit of work per `tsk.rulebook.md`'s Crate Scope Unity/Crate Locality principles, rather than
filing 3 near-identical micro-tasks for the same defect class.

**Related Tasks:** `062` (`task/completed/062_minwebgl_marker_resolution.md`) — its own I3/AF1
finding first identified site 1 (`geometry.rs`) but explicitly left it unfixed as out of that
verification pass's scope. AF1 there already specifies the exact re-check: `grep -n AsBytes
module/min/minwebgl/src/geometry.rs` must show more than 1 hit before that specific finding can be
marked resolved — satisfied automatically once this task deletes the dead import (0 hits after,
which is the correct "resolved by deletion" outcome, distinct from AF1's "resolved by adding a real
use" framing; either ending removes the drift).

## History

- **[2026-08-11]** `RESOLVED-IN-TREE` — All 5 dead imports are confirmed deleted in the current
  working tree, and the deletion is gate-proven. Fresh evidence this session:
  - `grep -n 'AsBytes\|VariantIterator\|IntoEnumIterator' src/{geometry,buffer,ubo}.rs` → exactly 2
    hits, both the *legitimate* fully-qualified uses (`buffer.rs:43` and `ubo.rs:16`, `Data :
    mem::AsBytes + ?Sized`); zero bare-import hits. `git diff` on the 3 files shows the uncommitted
    deletions: `geometry.rs` line 4 dropped `AsBytes` (and nothing else); `buffer.rs` line 4
    dropped `AsBytes` + `StrideTrait`; `ubo.rs` line 3 dropped `AsBytes`, `IntoEnumIterator`,
    `VariantIterator`.
  - **Attribution caveat:** this workspace hosts concurrent uncommitted work from the broader
    058-sweep effort; the deletions landed as part of the concurrent minwebgl sweep tranche (16
    files, +191/−101 in the crate's diff), not via a standalone execution of this task file. With
    zero commit history on these lines, per-actor attribution is not possible — recorded factually.
  - **Verification (this session, independent of whoever edited):** `cargo clippy -p minwebgl
    --no-deps --all-targets --all-features -- -D warnings` → **exit 0**, 51s
    (`module/min/minwebgl/-0001_longrun.log`). Under `-D warnings` any surviving `unused_imports`
    site would fail the gate — green is positive proof all 5 are gone and no new ones appeared.
  - Task 062's AF1 re-check contract satisfied by the "resolved by deletion" ending: `grep -n
    AsBytes module/min/minwebgl/src/geometry.rs` → 0 hits. Awaits independent
    verification/promotion per the task lifecycle.
- **[2026-08-11]** `FILED` — Filed via lightweight Draft capture
  (`tsk.rulebook.md § Core Procedures : Procedure - Draft Task`, PROC8) during this session's TA106
  out-of-scope-findings triage. Site 1 classified via `tsk.rulebook.md § Task File : Deduplication
  Search` as Case E (closed task 062 already names this exact site, but its own scope explicitly
  excludes fixing it). Sites 2-3 are a fresh discovery from this session's own direct `cargo check`
  output, confirmed via `grep -rl "buffer.rs.*AsBytes\|ubo.rs.*AsBytes\|VariantIterator" task/` to
  have no prior mention anywhere in `task/` outside raw, non-authoritative compiler-output log
  files (`task/unverified/-00NN_longrun.log`). Folded into this one task rather than filed
  separately: same crate, same defect class (dead imports), same trivial fix shape.

# Per-crate #[allow] justification sweep (decomposed from task 036)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📝 (Draft)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Continue task 036's justify-or-remove sweep over the remaining `#[allow]`/`#![allow]` attributes, one
crate at a time. Census as of 2026-08-10: **1905 sites workspace-wide** (task 036 resolved ufo.rs's 8 and
established the procedure). Execute per-crate, module/ crates first; each crate is an independently
completable increment.

**Per-crate procedure (proven on `primitive_generation/src/text/ufo.rs` in task 036):**

1. `grep -rn "#!\?\[ *allow(" <crate>/src` — inventory the crate's sites.
2. Check lint inheritance: crates WITHOUT `[lints] workspace = true` in Cargo.toml suppress lints that
   are mostly not even enabled — their allows are prime removal candidates, but consider adding the
   inheritance line first so the workspace policy actually applies (separate decision, surface to user
   if a crate looks deliberately opted out).
3. Remove the crate's unjustified blanket allows, run
   `longrun .launch dir::<workspace root> -- cargo clippy -p <crate> --all-targets --all-features`.
4. For each lint that actually fires: fix the code where mechanical (iterator forms, format inlining,
   redundant control flow); re-add as a **scoped item-level attribute with a one-line justification
   comment** only where the fix would be a real refactor (e.g. `too_many_lines` on a linear state
   machine). Lints that don't fire were stale — stay removed.
5. `cargo test -p <crate> --all-features` for behavior insurance.

**Census (top offenders; full recount at pickup — counts drift):**

| Crate | Sites | Inherits workspace lints? |
|-------|-------|---------------------------|
| module/helper/tiles_tools | 383 | yes |
| module/helper/renderer | 87 | yes |
| module/math/mdmath_core | 83 | **no** |
| module/helper/primitive_generation | 40 (48 − ufo.rs 8) | yes |
| module/min/minwebgl | 44 | yes |
| module/min/mingl | 44 | yes |
| module/math/ndarray_cg | 41 | **no** |
| module/helper/tilemap_scene | 38 | yes |
| module/min/minwebgpu | 32 | yes |
| module/helper/line_tools | 32 | yes |
| module/helper/gpu_hal | 28 | yes |
| module/helper/embroidery_tools | (in tail) | **no** |
| examples/* (27 of 30 not inheriting) | ~1000 across ~50 crates | mostly no |

**Examples tranche (lower priority, likely collapses):** example crates carry near-identical blanket
blocks (`implicit_return`, `min_ident_chars`, `std_instead_of_core`, ...) — a copy-pasted template.
Several of those lints are already centrally allowed-with-justification in `[workspace.lints.clippy]`
(Cargo.toml lines 71-98), so for inheriting examples the file-level copies are pure redundancy; for
non-inheriting ones the decision is template-level (adopt inheritance + delete the blocks), not
per-site. Resolve the template question once, then the examples tranche is mechanical.

## History

- **[2026-08-10]** `FILED` — Decomposed out of task 036 at pickup per that task's own decomposition
  note: 1905 sites across 102 crates is not one diff. Task 036 closed with the census, the inheritance
  map, and the concrete first instance (ufo.rs) executed; this successor carries the per-crate remainder.

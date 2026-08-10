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
| module/helper/tiles_tools | ~~383~~ ✅ swept 2026-08-10 → 37 justified | yes |
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
- **[2026-08-10]** `INCREMENT` — **tiles_tools swept: 460 → 38 matches** (health.md recipe; 37 real
  attribute lines — the 38th match is a doc-comment mention in `flowfield.rs:483`). Largest crate done.
  - **Stripped:** 449 file-level blanket allow lines across 18 files (lib.rs's 76-line wall + the
    copy-pasted test template blocks), boundary-asserted script.
  - **Machine fixes:** three `cargo clippy --fix` passes (`--lib` and `--tests` separately — with
    `--all-targets` the lib's fixes get skipped; one conflicting-fix site in `layout.rs::next` hand-
    rewritten to early-return guard form first after a full-batch rollback). ~330 sites: `#[must_use]`
    ×160, format inlining, lossless casts, iterator forms. Logs `-0054`…`-0057`.
  - **Manual fixes (~30 sites):** dead code deleted (`events.rs` `has_listeners`/`as_any`), 3 manual
    `Clone` impls → `*self` (keeps `System`/`Orientation` unbounded), 2 `IntoIterator` impls added for
    `&Grid2D`/`&mut Grid2D`, `type_complexity` fields aliased (`MovementRequestApply`, `StateHandler`),
    `movable : &Movable` → by-value, match arms merged, `single_match_else` → `if let`,
    `similar_names`/`min_ident_chars`-adjacent renames, literal separators, unused imports removed,
    `default_trait_access`/`useless_vec` in tests.
  - **Docs written:** all 43 flagged `missing_docs` sites cleared (a few were phantom — already
    documented in the uncommitted tree; the rest written: variant fields in `events.rs`/`game_systems.rs`
    restructured multi-line with per-field docs, 4 event structs, `IncompatibleVersion` fields) +
    17 `# Errors` sections (serialization 11, debug 3, ecs/world 3). Green `-D warnings` gate with
    `missing_docs` warn-on is the proof of completeness.
  - **Justified attrs kept (37):** lib.rs crate-level policy block ×7 (`missing_inline_in_public_items`,
    `exhaustive_structs`/`enums` = literal construction is the API contract, 4 cast lints = game-scale
    grid↔pixel math) · `debug.rs` file-level `format_push_string` (all 23 sites in its renderers) ·
    item-level `unused_self` ×10 (stubs, reasons name what the real impl will read), `dead_code` ×8
    (construction state / future passes), `cast_possible_truncation` ×2, `needless_pass_by_value` ×1,
    `similar_names` ×1 · test files ×7 (`float_cmp` ×4 exact-value asserts, casts ×3). Every attr
    carries a one-line reason comment.
  - **Style:** 69 `--fix`-added compact attrs normalized to `#[ must_use ]`/`#[ inline ]` in the 12
    house-style files; compact-style files left consistent.
  - **Verification:** `cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings` exit 0
    (log `-0060`; re-confirmed exit 0 after style normalization, log `-0062`); `cargo test -p tiles_tools
    --all-features` 285 passed / 0 failed across 10 targets incl. 40 doc tests (log `-0061`).
  - **Policy observation for the remaining crates:** `missing_inline_in_public_items` is workspace-warned
    yet was blanket-allowed by nearly every crate; same tension for `exhaustive_*` and the cast family.
    Candidate for central `[workspace.lints.clippy]` allows — a user decision, not taken unilaterally;
    until then each swept crate re-adds them crate-level with justification as done here.

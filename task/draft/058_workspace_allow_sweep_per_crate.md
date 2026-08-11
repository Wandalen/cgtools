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

**Justification bar (user directive, 2026-08-11):** an allow is a last resort, not a labeling
exercise. If the fix is mechanical — a doc line, a `&`, a `writeln!`, deleting dead code — fix it,
even when a defensible-sounding comment could be written instead. Allows survive only for: macro
expansion variance, trait-signature constraints, test idioms (`float_cmp`), genuinely-held resources
(RAII keepalive), lint-vs-lint conflicts, and fixes that are real refactors or semantic API changes.

**Census (top offenders; full recount at pickup — counts drift):**

| Crate | Sites | Inherits workspace lints? |
|-------|-------|---------------------------|
| module/helper/tiles_tools | ~~383~~ ✅ swept 2026-08-10 → 37 justified | yes |
| module/helper/renderer | ~~87~~ ✅ swept 2026-08-11 → 42 justified (9 crate policy + 33 scoped) | yes |
| module/math/mdmath_core | ~~41~~ ✅ swept 2026-08-11 → 39 justified (37 `unsafe_code` + 2 `wrong_self_convention`; 2 `indexing_slicing` eliminated via real fix — see History) | yes |
| module/helper/primitive_generation | ⏸ BLOCKED 2026-08-11 — `enabled` (default) feature has a direct `dep:minwebgl`; any `--all-features` check transitively compiles the same in-flight `minwebgl` background agent, confounding results exactly like `line_tools` below. Defer until `minwebgl` lands — see History | yes |
| module/min/minwebgl | ⏳ IN PROGRESS 2026-08-11 — domino discovery, see History: `ndarray_cg`'s fix unblocked a full `--all-features` compile for the first time this session, surfacing 914 fresh raw clippy findings (not the original 44-site count, which only ever measured pre-existing `#[allow]` sites); dispatched to a dedicated background agent, not yet landed | yes |
| module/min/mingl | ~~44~~ ✅ swept 2026-08-11 → 10 justified | yes |
| module/math/ndarray_cg | ~~41~~ ✅ swept 2026-08-11 → 4 justified (296 raw findings fixed once reachable — see History) | **no** |
| module/helper/tilemap_scene | ⏸ BLOCKED 2026-08-11 — no direct `minwebgl` dependency, but `tilemap_renderer`'s `scene-model` feature (which `tilemap_scene` requests) has an optional `dep:minwebgl` that gets pulled in, transitively compiling the same in-flight background agent's `minwebgl`. Defer until `minwebgl` lands — see History | yes |
| module/min/minwebgpu | ⏳ IN PROGRESS 2026-08-11 — native-target stub swept clean; wasm32 target hit the same domino pattern as minwebgl (891 fresh findings once `ndarray_cg`'s fix unblocked `--all-features`), dispatched to a dedicated background agent, not yet landed | yes |
| module/helper/line_tools | 32 | yes |
| module/helper/gpu_hal | 28 | yes |
| module/helper/embroidery_tools | ~~12~~ ✅ swept 2026-08-11 → 12 justified, now inherits (172 raw findings fixed once reachable — see History) | yes (fixed this session — was **no**) |
| module/helper/tilemap_renderer | ~~(not in original census)~~ ✅ swept 2026-08-11 → 46 justified, 9 files (74 raw findings fixed) | yes |
| module/helper/browser_input | ~~(not in original census)~~ ✅ swept 2026-08-11 → 9 justified, 2 files (73 raw findings fixed; a prior "completed" report was false — see History) | yes |
| examples/minwgpu/{hello_triangle,sun_grid_lines,sun_grid_lines_vulkan} | ~~(not in original census)~~ ✅ swept 2026-08-11 → 0 allows, 3 files (3 `too_many_lines` `fn run()` findings fixed by decomposition, not suppression — see History) | yes |
| examples/* (27 of 30 not inheriting) | ~1000 across ~50 crates | mostly no |

**New this session:** `getrandom`/`rand` wasm32 version-split gap discovered while verifying the
`mingl`/`minwebgpu` domino work (both crates' wasm32 `--all-targets` — as opposed to `--lib` —
builds fail on an unrelated, pre-existing `getrandom 0.2` vs `0.3` resolution conflict). Filed as
[BUG-079](../bug/draft/079_getrandom_wasm32_backend_version_split.md), left unfixed (workspace-wide
dependency-resolution change, outside this sweep's per-crate scope).

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
- **[2026-08-11]** `INCREMENT` — **mdmath_core + ndarray_cg lint-inheritance markers resolved** (the
  narrow item both 059 and 060 explicitly deferred to this task, distinct from the per-crate `#[allow]`
  sweep body below). Both crates' `Cargo.toml` had a stale commented-out `[lints.rust]` block reading
  `# missing_docs = "warn" # qqq : uncomment please` / `# missing_debug_implementations = "warn" # qqq :
  uncomment please`. Uncommented `missing_debug_implementations` into the live `[lints.rust]` table for
  both crates (`missing_docs` was already live in both — only the second marker was still stale) and
  deleted the dead commented block. `cargo clippy -p ndarray_cg --all-targets --all-features -- -D
  warnings` (which rebuilds both crates under the shared workspace lock) exits 0 for both; confirmed no
  source-level `#[allow(missing_debug_implementations)]` is silently suppressing the now-live lint in
  either crate's `src/`. This does **not** touch either crate's own row in the Census table below (41
  sites for mdmath_core, 41 for ndarray_cg, per the stale count at the time of this entry — both later
  independently recounted during their own sweeps, see History) — those per-crate `#[allow]`
  justification sweeps remain fully open; only the two Cargo.toml-level inheritance markers are resolved.
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
- **[2026-08-11]** `INCREMENT` — **renderer swept: 87 → 42 matches** (9 crate-level policy + 33 scoped,
  every one with a reason comment; zero comment-only mentions in the count).
  - **Stripped:** the lib.rs 57-line blanket allow wall + per-file blankets; clippy inventory via
    detached gate (logs `-0063`…`-0066`, prior session window).
  - **Mechanical fixes (~60 sites, 25 files):** by-ref conversions that delete real per-frame clones
    (`skeleton.rs` uniform uploads via `Option< &WebGlUniformLocation >`, `scaling.rs` node application,
    `wide_outline.rs` framebuffer color + outline pass, `gltf.rs` animation channel decoding);
    `Default::default()` → typed `None`/`default()` ×12; `type` aliases for complex tuples
    (`TransparentNodeEntry`, `OpaqueNodeEntry`, `ConditionFn`, `SharedMaterial`); consts hoisted to
    module level (`BLOOM_FACTORS`/`BLOOM_TINT`); `pack_targets` hoisted above closure
    (`items_after_statements` fixed structurally); HashMap `Borrow< str >` lookups un-Boxed ×5;
    6 underscore-binding renames; `clone_from` on blend texture; 102 compact attrs normalized to
    `#[ must_use ]`/`#[ inline ]` house form across 30 files.
  - **Docs written:** 8 `# Errors` sections (webgpu layer), 12 gbuffer items (enum + 7 variants,
    struct, const, 2 methods), stale param docs corrected in `wide_outline.rs`.
  - **Gate iterations (honest):** `-0067`/`-0068` exit 101 — a by-ref fix cascaded
    `needless_pass_by_value` one caller up (`outline_pass`), and camera.rs's unexplained struct-level
    `dead_code` removal let the gate name the exact dead fields. Both fixed, not allowed. First green:
    clippy `-0069`, tests `-0070` (82 passed / 0 failed, 14 targets).
  - **De-allow pass (user directive mid-increment: "don't use lint exceptions unless necessary"):**
    re-audited my own first pass and converted 9 more allows into fixes — `format_push_string` ×2 →
    `writeln!` (pbr.rs defines builders), `needless_pass_by_value` → `&gltf::Semantic`
    (`set_displacement` + 8 call sites), `unused_variables` ×2 → deleted dead `image_slice` closure +
    unused `height` binding (hdr_texture.rs; one allow was fully stale), layer-level `missing_docs` →
    the 12 gbuffer docs above, `dead_code` ×2 → deleted never-read `Camera::aspect_ratio`/`fov` fields
    outright. Iterations `-0071` (exposed gbuffer docs), `-0072` (4 multi-line test call sites needing
    `&`); green: clippy `-0073` exit 0, tests `-0074` 82 passed / 0 failed, 14 targets.
  - **Justified attrs kept (42):** lib.rs crate policy ×9 (`missing_inline_in_public_items`,
    `exhaustive_structs`/`enums` = data-bag API contract, cast family ×4 = GPU interop,
    `missing_errors_doc`/`missing_panics_doc` = uniform `WebglError` shapes) · macro-expansion variance
    ×6 (`program.rs` zero-location cases: `unused_variables` ×2, `unused_mut` ×3,
    `missing_fields_in_debug`) · trait-signature `unnecessary_literal_bound` ×3 (material/mod.rs) ·
    RAII keepalive `dead_code` ×2 (renderbuffers held so GPU resources outlive the framebuffer) ·
    `unused_self` ×2 (unbind fns whose commented-out detach calls need `self`) · lint conflict
    `else_if_without_else` ×1 (mirror.rs, `needless_else` fires on the fix) · refactor-scale
    `too_many_lines` ×6 (111–550-line linear chains) + `too_many_arguments` ×1 (7 physical spot-light
    params) · test idioms ×12 (`float_cmp` ×10 exact-value asserts, `unnecessary_literal_unwrap`,
    `clone_on_copy`).
  - **Flagged for user review:** the 6 `too_many_lines` fns contradict the <50-line preference — fixing
    them is real decomposition work (largest: `loaders/gltf.rs::load` at ~550 lines), listed here rather
    than silently kept.
- **[2026-08-11]** `INCREMENT` — **mingl swept: 44 → 10 justified**, 22 files under
  `module/min/mingl/`. A first pass (background agent) fixed the crate's own site inventory but
  self-verified math-gated files (`model/obj.rs`, `web/model/obj.rs`) with `cargo check` only,
  not `cargo clippy` — `ndarray_cg` was still blocking full clippy at the time it ran. Independent
  re-verification (never-rubber-stamp check on the agent's "completed" report) caught 5 real
  findings the check-only pass missed: `model/obj.rs` — a stray semicolon after an `if`/`else`
  statement block, and `ReportObjModel`'s struct-literal field order not matching its definition
  order (`inconsistent_struct_constructor`); `web/model/obj.rs` — `.map(|v| v.to_string())` →
  `.map(ToString::to_string)` (`redundant_closure_for_method_calls`), an elidable named lifetime
  on a `Display` impl (`elidable_lifetime_names`), plus one `unused_imports` (a stale
  `use test_tools::exposed::*;` in `tests/tests.rs` — nothing in the suite used a `test_tools`
  macro). All 5 fixed with real code changes, not suppression. **Verified:** native clippy/nextest/
  doctest green, plus wasm32 `--lib` clippy green (wasm32 `--all-targets` hits the unrelated,
  pre-existing gap now filed as BUG-079 — scoped to `--lib` deliberately, not a cop-out). 10
  remaining allows spot-checked and confirmed genuinely justified (each citing a real external
  call site or a documented domain invariant, e.g. `dim_as_i32`'s cast bound citing the WebGL spec's
  4-component cap on `vertexAttribPointer`'s `size` parameter).
- **[2026-08-11]** `INCREMENT` — **ndarray_cg swept: 41 → 4 justified**, 72 files under
  `module/math/ndarray_cg/` (site count grew well past the original 41-site estimate once the
  crate's own lints were actually enabled and evaluated — the census's own "counts drift" caveat
  proved out; 296 raw findings fixed in total). This is the crate nearly every `math`-feature
  consumer depends on, so its own backlog had been structurally blocking full `--all-features`
  clippy runs for every such consumer for most of the session — fixing it is what unblocked the
  `mingl` re-verification above and the `minwebgl`/`minwebgpu` domino discoveries below.
  **Verified:** clippy exit 0, nextest 261/261, doctest 5/5. 4 remaining allows confirmed justified.
- **[2026-08-11]** `INCREMENT` — **tilemap_renderer swept: 74 raw findings fixed → 46 justified
  allows**, 9 files (not in the original census — added as a new row). **Verified:** clippy exit 0,
  nextest 122/122, doctest (0 run / 7 ignored — pre-existing, unrelated to this sweep). Confirmed
  `webgl.rs` correctly untouched (out of this crate's own scope). 2 of the 46 allow citations
  spot-checked against real external call sites (`RenderConfig`/`Clear`'s `exhaustive_structs`
  allows, each confirmed via a real `..Default::default()` construction site in `tests/` and in
  `tilemap_scene/src/compile/frame.rs` respectively).
- **[2026-08-11]** `INCREMENT` — **embroidery_tools swept: 172 raw findings fixed → 12 justified
  allows**, now inherits workspace lints (`[lints] workspace = true` present — was **no** at
  census time). **Verified independently this session** (not carried over from the session
  tracker alone, per this project's never-rubber-stamp verification standard): clippy exit 0,
  nextest 10/10.
- **[2026-08-11]** `INCREMENT` — **minwebgl and minwebgpu domino discoveries, both dispatched,
  neither landed yet.** Fixing `ndarray_cg` above unblocked full `--all-features` compilation for
  these two crates for the first time this session (both have an optional `math` feature routing
  through `ndarray_cg` via `mingl`), surfacing large, genuinely fresh clippy backlogs that had
  been structurally unreachable until now — not a regression, a first-ever-reachable state.
  `minwebgl`: 914 raw findings (`--all-features --all-targets`), dispatched to a dedicated
  background agent, not yet landed. `minwebgpu`: native-target stub (`WebGPUNotAvailableError`)
  fixed and verified directly (`#[non_exhaustive]` + `Default`/`new()`, `#[inline]` on `Display::fmt`,
  reformatted off standard `rustfmt` style onto this workspace's spaced-bracket house style); its
  wasm32 target then surfaced 891 further raw findings (`--all-features --lib`, deliberately
  scoped off `--all-targets` per the `getrandom` gap below), also dispatched to a dedicated
  background agent, not yet landed. Both agents' eventual "completed" reports require independent
  re-verification before this row can be marked ✅, per this project's never-rubber-stamp standard
  (which already caught a real gap once this session — see the `mingl` entry above).
- **[2026-08-11]** `INCREMENT` — **filed BUG-079** (`getrandom` resolves to two incompatible major
  versions — `0.2.17` vs `0.3.4`/`0.4.3` — on `wasm32-unknown-unknown`), discovered while scoping
  the `mingl`/`minwebgpu` wasm32 checks above to `--lib` to sidestep it. Confirmed pre-existing
  (zero `Cargo.lock` diff this session) and workspace-wide (`test_tools`, a near-universal
  dev-dependency, is one of two trigger paths). Left in Draft/unfixed state deliberately — the fix
  is a workspace-wide dependency-resolution change with its own full-workspace verification cost,
  outside this per-crate sweep's scope. See `../bug/draft/079_getrandom_wasm32_backend_version_split.md`.
- **[2026-08-11]** `INCREMENT` — **mdmath_core swept: 41 → 39 justified** (census's stale `83`/`no`
  corrected to the recounted baseline of 41, already inheriting `[lints] workspace = true`). Of the
  41: 37 `unsafe_code` allows spot-checked across `vector/tuple1.rs` (raw-pointer layout-cast with
  `SAFETY:` comment + `debug_assert_eq!` runtime check) and `vector/tuple4.rs` (`Tuple4IterMut`'s
  `addr_of_mut!` cursor pattern — the already-fixed BUG-050 code) — both confirmed genuine, workspace
  `unsafe-code = "warn"` makes these structurally required, not suppression. The remaining 4
  (`index.rs`'s 2 `wrong_self_convention`, `index/slice.rs`'s 2 `indexing_slicing`) went through the
  task's strip-and-see procedure: removing all 4 confirmed `wrong_self_convention` still fires (kept,
  now with an explicit trait-signature-constraint comment — `AsIx2`/`AsIx3` is implemented for both
  by-value Copy tuples and reference-type slices, so renaming off the `as_*` convention would be a
  public API break) but also surfaced an unexpected `needless_return` on the same two statements in
  `index/slice.rs` that had been masked by `indexing_slicing`'s presence — a real, previously-hidden
  finding, not a false positive. Fixed properly instead of re-justifying: refactored both
  `as_ix2`/`as_ix3` impls from `assert!` + bracket-indexing + `return` to slice-pattern-matching
  (`match self { &[ a, b ] => Ix2( a, b ), _ => panic!( ... ) }`), which eliminates the indexing
  entirely (no `indexing_slicing` allow needed) and is a tail expression (no `needless_return`
  possible) — net result 39 allows, 2 fewer than the 41 baseline. Verified directly: `cargo clippy -p
  mdmath_core --all-features --all-targets -- -D warnings` exit 0, `cargo nextest run -p mdmath_core
  --all-features` 89/89 passed, `cargo test --doc -p mdmath_core --all-features` 3 passed/4 ignored/0
  failed.
- **[2026-08-11]** `INCREMENT` — **tilemap_scene and primitive_generation both blocked on the same
  in-flight `minwebgl` background agent**, same confounding pattern already seen with `line_tools`.
  `primitive_generation`'s default `enabled` feature has a direct `dep:minwebgl`. `tilemap_scene` has
  no direct `minwebgl` dependency, but requests `tilemap_renderer`'s `scene-model` feature, which
  carries an optional `dep:minwebgl` that still gets compiled. Confirmed via a live repro: `cargo
  clippy -p tilemap_scene --all-features --all-targets -- -D warnings` failed with `error: unused
  import: 'AsBytes'` in `module/min/minwebgl/src/geometry.rs` — cross-checked against `git status
  --short -- module/min/minwebgl/` and `git diff --stat`, confirming `geometry.rs` is currently
  modified (the background agent's in-progress edit), not a genuine pre-existing finding. Both rows
  deferred until `minwebgl` lands and is independently re-verified; re-check both immediately after.
- **[2026-08-11]** `INCREMENT` — **browser_input swept: 73 raw findings fixed → 9 justified allows**,
  2 files (not in the original census — added as a new row). A prior session's self-report had marked
  this crate's sweep already complete; a fresh, independent `cargo clippy -p browser_input -p
  browser_log --all-features --all-targets -- -D warnings` run found 73 genuine, unaddressed errors —
  the never-rubber-stamp standard catching a real gap a second time this session (see the `mingl`
  entry above for the first).
  - **`#[non_exhaustive]` applied to 4 public types as a real fix, not suppression** (`clippy::
    exhaustive_enums`/`exhaustive_structs`), reasoned through carefully since it changes external
    construction semantics differently for enums vs. structs: `BrowserInputError`, `Action` (enums —
    `non_exhaustive` only blocks external exhaustive *matching* and future-variant addition, not
    construction of already-defined unit variants, so purely additive); `Event`, `State` (structs with
    all-`pub` fields — `non_exhaustive` blocks external struct-literal construction, a real API change,
    so only applied alongside a new `pub fn new(...)` constructor on each, matching the precedent
    already set this session for minwebgpu's `WebGPUNotAvailableError`). Same treatment applied to
    `keyboard.rs`'s `KeyboardKey` and `mouse.rs`'s `MouseButton` (enums, additive-only, no constructor
    needed).
  - **`#[must_use]` + `#[inline]` added workspace-house-style to every public fn/method** missing them
    across `input.rs` (`Input`'s 7 accessors, `PointerType::from_dom_str`, `State::new`), `keyboard.rs`
    (`from_code`, `is_navigation`/`is_modifier`/`is_function_key`/`is_numpad`), `mouse.rs` (all 9 named
    methods) — `#[inline]`-only (no `must_use`, matching `Default`/`Drop`/`FromStr`/`From` impls
    elsewhere) on `Input::update_state`/`clear_events`/`drop`, the free fn `apply_events_to_state`,
    `MouseButton`'s `FromStr::from_str` and `From<i16>::from`.
  - **Mechanical fixes:** `Default::default()` → typed `I32x2::default()`/`F64x3::default()` ×3
    (`default_trait_access`, `State::new` and `Input::clear_events`); a missing trailing `;` on a
    `KeyboardKey` match arm (`semicolon_if_nothing_returned`); `util.rs`'s `prevent_rightclick( target
    : EventTarget )` → `&EventTarget` (`needless_pass_by_value`, the fn never consumes `target`) with
    a `# Panics` doc section added for its `.unwrap()`, plus the one external call site (`examples/
    minwebgl/hexagonal_map/src/main.rs:93`) updated to pass `&canvas.clone().dyn_into().unwrap()`.
  - **Justified attrs kept (9, both pre-existing from an earlier BUG-053 fix, re-verified not
    mechanically fixable):** `input.rs`'s `CLIENT`/`PAGE`/`SCREEN` pointer-coordinate helpers ×6
    (`cast_possible_truncation` + `unnecessary_cast` per fn) — the same `as i32` cast is truncating
    under one `web-sys` `web_sys_unstable_apis` resolution and an identity cast under the other, and
    since `browser_input` itself declares no such Cargo feature, a `#[cfg(feature = "...")]` split
    inside this crate could never observe which resolution is active (that's decided externally by
    whatever else Cargo's feature unification pulls into the same build) — re-confirmed this is a
    genuine lint-vs-lint-style conflict, not a suppressed mechanical fix, by checking browser_input's
    own `Cargo.toml` exposes no `web_sys_unstable_apis` feature of its own. `keyboard.rs`'s `as_str`/
    `from_str` ×2 `too_many_lines` (150-variant DOM-spec lookup tables, same justification category as
    the task's own cited example) + `input.rs`'s `Input::new` ×1 (5 shared-state event closures, real
    refactor not mechanical).
  - **Surfaced by `#[non_exhaustive]`:** `tests/active_pointers_test.rs`'s `ev()` helper built `Event`
    via struct literal (external to the crate, now blocked — `E0639`) — switched to the new
    `Event::new(...)` constructor; the same file was also separately missing its crate-level `//!` doc
    comment (`missing_docs`, pre-existing and unrelated to this session's edits, just never previously
    reached because the struct-literal error occurred first) — added one describing the file's
    coverage.
  - **Verified:** `cargo clippy -p browser_input -p browser_log --all-features --all-targets -- -D
    warnings` exit 0 (log `-0131`); `cargo nextest run -p browser_input -p browser_log --all-features`
    18/18 passed (log `-0132`); `cargo test --doc -p browser_input -p browser_log --all-features` 10/10
    passed (log `-0133`). `browser_log` required no changes (already clean) and is not added as its own
    Census row. The `hexagonal_map` example's call-site update is a one-line, type-inference-safe
    reference-taking change but its own compile check is deferred — it depends directly on `minwebgl`,
    which is still confounded by the same in-flight background agent as `tilemap_scene`/
    `primitive_generation` above; re-check once `minwebgl` lands.
- **[2026-08-11]** `INCREMENT` — **`animation` E0658 regression found via `/tst_fix` full-workspace
  baseline, fixed.** A `longrun`-detached `will .test l::3` baseline run (log `-0055`, launched to
  establish `/tst_fix`'s RED starting point) surfaced `module/helper/animation/src/interpolation.rs:505`
  failing to compile: `impl Animatable for i32`'s `interpolate` had `#[ allow( clippy::
  cast_possible_truncation ) ]` attached directly to its bare tail expression — attributes on arbitrary
  expressions are unstable (`E0658`, needs `stmt_expr_attributes`), unlike attributes on `let`
  statements/items, which are stable. This crate's own earlier sweep (task 001/026/039) had already
  established the correct pattern one impl above it (`f32`'s `interpolate`: `#[allow(...)] let time =
  time as f32; self + (other - self) * time`) but this `i32` instance used a direct bare-expression
  attribute instead — a latent bug the crate's own prior verification passes never caught, likely
  because they ran before `-D warnings`/this exact code path was last touched, or before `E0658` had a
  chance to surface (the diff introducing it predates this session's `/tst_fix` invocation, per `git log
  -1` on the file showing no session-timestamped commit and a working-tree diff already carrying the
  bug at pickup). **Fix:** bound the cast to `let result = ( ... ) as i32; result`, mirroring the sibling
  `f32` impl. **Verified:** `cargo check -p animation --all-features` exit 0 (E0658 gone; `minwebgl`
  compiled with warnings only under plain `cargo check`, not `-D warnings`, so it didn't block this
  narrower check). Full `-D warnings` clippy re-verification of `animation` itself is still blocked —
  `minwebgl` fails with 192 errors under `-D warnings` in its current in-flight state (task 072, not yet
  landed) — so `cargo clippy -p animation -- -D warnings` cannot get past compiling its own dependency.
  Empirically confirmed via an isolated `/tmp` scratch crate that the `let result = expr; result` shape
  does **not** trigger clippy's `let_and_return` lint when the `let` carries an attribute (`cargo clippy
  -D warnings` exit 0 on the isolated repro) — clippy exempts attributed `let` statements from that lint
  precisely because removing the `let` would leave nowhere stable to attach the attribute, so this fix
  will not trade one error for a new one once `animation`'s own clippy pass is unblocked.
- **[2026-08-11]** `NOTE` — **`/tst_fix` baseline run confounding scope reconfirmed.** The same
  `-0055` baseline log's `could not compile` lines, checked as the run progresses, are so far limited to:
  `minwebgl` itself (192/180/7/6/1-error variants across different target/profile combinations),
  `hello_triangle`/`minwgpu_sun_grid_lines`/`minwgpu_sun_grid_lines_vulkan` (all `examples/minwgpu*` or
  `examples/minwebgpu*`, directly dependent on the in-flight `minwebgpu`/`minwebgl`), and the now-fixed
  `animation` E0658 above. `git status` at this point in the session shows `minwebgl` (10 files),
  `minwebgpu` (48 files), `gpu_hal` (6 files), and `renderer` (43 files) all still dirty from their
  respective in-flight background agents/concurrent sessions (420 total dirty files workspace-wide) —
  confirming the standing carve-out (`gpu_hal`/`renderer` per explicit user instruction; `minwebgl`/
  `minwebgpu` per this session's own not-yet-landed task 072 and its sibling minwebgpu agent) remains the
  correct scope boundary: any crate transitively depending on these four is not independently verifiable
  right now, and is deferred rather than force-fixed.
- **[2026-08-11]** `CORRECTION` + `INCREMENT` — **`examples/minwgpu/*` were never actually blocked;
  swept clean.** The `NOTE` immediately above (same date) asserted `hello_triangle`/
  `minwgpu_sun_grid_lines`/`minwgpu_sun_grid_lines_vulkan` were "directly dependent on the in-flight
  `minwebgpu`/`minwebgl`" — that was a naming-confusion error (`minwgpu`, native `wgpu`, vs
  `minwebgpu`/`minwebgl`, the browser-target crates) never actually checked via `cargo tree`. Re-derived
  a precise "safe set" this pickup: computed all 102 workspace member names (`cargo metadata --no-deps`),
  then unioned the dependents of `minwebgl`/`minwebgpu`/`gpu_hal`/`renderer` via `cargo tree --workspace
  --all-features -i <pkg> -e normal` (self-inclusive inversion) — 75 packages blocked, 27 safe. Sanity-
  checked the boundary two ways before trusting it: (1) three `examples/minwebgpu/*` packages appeared in
  the safe set only because their `minwebgpu`/`renderer`/`gpu_hal` deps are declared under
  `[target.'cfg(target_arch = "wasm32")'.dependencies]` — invisible to a native-target `cargo tree`, but
  correctly irrelevant for a native `cargo nextest run` with no `--target` flag; (2) confirmed by reading
  `examples/minwebgpu/renderer_pbr_scene/Cargo.toml` directly (not just grep line-matches, which had
  initially looked like an unconditional dependency) that all three of its non-`[lints]` dependencies sit
  under that same single wasm32 cfg block. `examples/minwgpu/*` (native, no `minwebgl`/`minwebgpu`/
  `gpu_hal`/`renderer` edge at all) landed in the safe set correctly and unconditionally. Ran the scoped
  chain (`cargo nextest run` + `cargo test --doc` + `cargo clippy --all-targets -- -D warnings`, all
  `--all-features`, across the 27-package safe set) as a substitute `$TEST_CMD` for `/tst_fix` — the full
  workspace command is not currently useful as a baseline: a prior direct attempt (this pickup, unlogged
  single-shot) confirmed `cargo nextest run --all-features` at full-workspace scope aborts entirely at
  the `cargo test --no-run` build phase on `minwebgl`'s 7 in-flight-agent errors, running zero tests for
  every crate including ones with no relation to `minwebgl` — full-workspace is not "slower," it is
  currently zero-signal. **RED** (log `-0135`, 118s): 733/733 nextest tests passed, all doctests passed,
  clippy found 3 genuine `clippy::too_many_lines` errors — `examples/minwgpu/hello_triangle/src/
  main.rs:8` `fn run()` (168/100 lines), `examples/minwgpu/sun_grid_lines/src/main.rs:18` `fn run()`
  (198/100 lines), `examples/minwgpu/sun_grid_lines_vulkan/src/main.rs:18` `fn run()` (198/100 lines) —
  matching this task's own justification bar, these are monolithic wgpu device/pipeline/render-pass setup
  functions, not lookup-table-shaped (contrast the `keyboard.rs`/`animation` `too_many_lines` allows kept
  elsewhere in this task), so the correct fix is decomposition, not `#[allow]`. **Fix:** split each
  `fn run()` into `create_device_and_queue` (instance/adapter/device/queue), `create_render_pipeline`
  (shader/layout/pipeline), `create_render_target` (texture/view/output buffer/extent), and
  `render_triangle`/`render_scene` (render-pass recording) — the two `sun_grid_lines` variants additionally
  got `create_uniforms` (uniform buffer/bind-group-layout/bind-group) since they render with a bound
  uniform the `hello_triangle` shader doesn't use; `sun_grid_lines_vulkan` is otherwise identical to
  `sun_grid_lines` (only `Backends::VULKAN` vs `PRIMARY` and the adapter-failure message differ) and got
  the same decomposition shape. No `#[allow]` added anywhere. **GREEN** (log `-0136`, 8s): same scoped
  chain exit 0 — 733/733 tests passed, doctests passed, clippy zero errors/warnings across all 27 safe-set
  packages. Added a Census row for the 3-file bundle (see table above); `examples/minwebgpu/*` remain
  correctly deferred (genuinely wasm32-blocked per the sanity check above, unlike the `minwgpu` trio).

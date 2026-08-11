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
   `longrun .launch dir::<workspace root> -- cargo clippy -p <crate> --no-deps --all-targets --all-features -- -D warnings`.
   `--no-deps` is mandatory: without it the trailing `-D warnings` reaches every workspace-member
   dependency via CLIPPY_ARGS and drags unswept crates' reasonless allows into this crate's gate.
4. For each lint that actually fires: fix the code where mechanical (iterator forms, format inlining,
   redundant control flow); re-add as **`#[ expect( lint, reason = "..." ) ]`** only where the fix
   would be a real refactor (e.g. `too_many_lines` on a linear state machine) — `expect` self-detects
   staleness via `unfulfilled_lint_expectations`. Use `#[ allow( lint, reason = "..." ) ]` only where
   `expect` structurally can't work: `dead_code` (rustc #114557 — detection treats `expect` as `allow`,
   so the expectation never fulfills) and per-expansion macro-body sites. Lint attrs on a macro
   *invocation* are ignored outright by rustc (`unused attribute`, both `expect` and `allow`) — hoist
   those to a file-level inner attr instead (e.g. criterion_group!'s generated `pub fn`). Families
   noisy by design (casts ×4, `missing_inline_in_public_items`, `exhaustive_enums`/`structs`) are
   centrally allowed in root `Cargo.toml` — DELETE scoped copies rather than converting them; the
   `allow_attributes_without_reason = "warn"` ratchet rejects any new reasonless suppression.
   Lints that don't fire were stale — stay removed.
5. `cargo test -p <crate> --all-features` for behavior insurance.

**Justification bar (user directive, 2026-08-11):** an allow is a last resort, not a labeling
exercise. If the fix is mechanical — a doc line, a `&`, a `writeln!`, deleting dead code — fix it,
even when a defensible-sounding comment could be written instead. Allows survive only for: macro
expansion variance, trait-signature constraints, test idioms (`float_cmp`), genuinely-held resources
(RAII keepalive), lint-vs-lint conflicts, and fixes that are real refactors or semantic API changes.

**Census (top offenders; full recount at pickup — counts drift):**

| Crate | Sites | Inherits workspace lints? |
|-------|-------|---------------------------|
| module/helper/tiles_tools | ~~383~~ ✅ swept 2026-08-10; re-greened under expect regime 2026-08-11 → 18 justified (15 expect + 3 allow) | yes |
| module/helper/renderer | ~~87~~ ✅ swept 2026-08-11 → ~~33~~ 27 justified scoped (20 expect + 7 allow); crate policy block deleted — families now central; all 6 `too_many_lines` expects eliminated via proper decomposition 2026-08-11 — see History | yes |
| module/math/mdmath_core | ~~41~~ ✅ swept 2026-08-11 → ~~39~~ 44 justified, all machine-checked expects (37 `unsafe_code` + 2 `wrong_self_convention` + 5 test `clippy::float_cmp`; comment-only justifications converted to `expect` in the 2026-08-11 residue tranche, fulfillment gate-proven; 2 `indexing_slicing` eliminated via real fix — see History) | yes |
| module/helper/primitive_generation | ~~⏸ BLOCKED~~ ✅ swept 2026-08-11 → 3 justified (all expect: float_cmp coincident-point guard, 2× too_many_lines) — 6 pre-unblock findings + 47 surfaced by `--all-features` once minwebgl landed, all real fixes; stale test-file cast allow deleted, ufo.rs too_many_lines allow → expect-with-reason; 6 downstream example call sites updated across 4 crates (breaking-API cleanup) — see History | yes |
| module/min/minwebgl | ~~44~~ ~~⏳ IN PROGRESS~~ ✅ landed + verified 2026-08-11 → 11 justified (all reasoned allows): the dispatched 914-finding tranche landed and was verified by this task's independent gates (host `--all-targets --all-features` + wasm32 `--lib`); residue: 6 stale central-family duplicates deleted, `unexpected_cfgs` allow replaced by workspace `check-cfg` declaration for `web_sys_unstable_apis` — see History | yes |
| module/min/mingl | ~~44~~ ✅ swept 2026-08-11 → ~~10~~ 1 justified (residue tranche 2026-08-11: 10 stale central-family duplicates deleted, cast-safety comments kept; the 1 survivor is obj.rs `deprecated` → expect — see History) | yes |
| module/math/ndarray_cg | ~~41~~ ✅ swept 2026-08-11 → ~~4~~ 22 justified, all machine-checked expects (8 `unsafe_code` + 11 test `clippy::float_cmp` + 2 `needless_pass_by_value` + 1 `op_ref`; residue tranche 2026-08-11 converted comment-only justifications to `expect` and deleted 2 stale `exhaustive_structs` duplicates; 296 raw findings fixed once reachable — see History) | ~~**no**~~ yes — committed manifest carries `[lints] workspace = true`, stale flag corrected 2026-08-11 |
| module/helper/tilemap_scene | ~~⏸ BLOCKED~~ ✅ swept 2026-08-11 → 15 justified (5 expect + 10 allow-with-reason; tests/common dead_code allows documented per-binary) — 35 findings across lib + 7 test binaries, all real fixes (project_to_transform dedup, sampler-type re-exports, file-level float_cmp expects); suite 169/169 — see History | yes |
| module/min/minwebgpu | ~~32~~ ~~⏳ IN PROGRESS~~ ✅ landed + verified 2026-08-11 → 0 attrs: wasm32 `--lib` + host gates green (3 findings fixed here incl. a real copy-paste bug — compute_pipeline.rs `map_err` built `FailedToCreateRenderPipeline` instead of `FailedToCreateComputePipeline`); 6 stale cast_precision_loss duplicates deleted; wasm32 `--all-targets` BUG-079 block RESOLVED 2026-08-11 (target-gated getrandom 0.2 `js` shim; per-crate `--all-features --all-targets` `-D warnings` gate green, ledger `-0261`) — see History | yes |
| module/helper/line_tools | ~~32~~ ✅ swept 2026-08-11 → 4 justified (all expect) — census was stale: crate had 0 attrs and 151 latent lib errors | yes |
| module/helper/gpu_hal | ~~28~~ ✅ swept 2026-08-11 → 7 justified (6 allow-with-reason, combo-dependent + 1 cfg-gated expect); 14 stale suppressions deleted via expect-flip; wasm32-context lint debt (16 findings: 15 `must_use`, 1 `# Errors`, 1 `match_same_arms`) adopted + fixed 2026-08-11 after the concurrent lane's conclusion, both-target clippy green — see History | yes |
| module/helper/embroidery_tools | ~~12~~ ✅ swept 2026-08-11 → ~~12~~ 0 attrs (residue tranche 2026-08-11: all 12 were stale central-family duplicates — `exhaustive_structs`/cast family — deleted, cast-safety comments kept; 172 raw findings fixed once reachable — see History) | yes (fixed this session — was **no**) |
| module/helper/tilemap_renderer | ~~(not in original census)~~ ~~⚠️ 40 sites needing `reason=`~~ ✅ re-swept 2026-08-11 → 1 justified expect in src/ minus webgl.rs (svg.rs `collapsible_match`, E0004 exhaustiveness rationale), 0 in tests/: the 40 debt sites were all `#[ allow( clippy::exhaustive_structs ) ]` — centrally allowed family, deleted as stale duplicates rather than given reasons; +4 more stale feature-gated duplicates deleted in svg.rs (casts ×2 attrs, `std_instead_of_core` ×2), +2 test files' crate docs moved above `#![ cfg ]` (latent `missing_docs` when the feature is off); `adapters/webgl.rs` backlog closed 2026-08-11: fixed in the concurrent minwebgl-tranche work, independently verified by this task (`enabled,adapter-webgl` + `--all-features` clippy green, suite 128/128 all-features); webgl.rs now 4 reasoned allows, 0 reasonless — see History | yes |
| module/helper/browser_input | ~~(not in original census)~~ ✅ swept 2026-08-11 → ~~9~~ 6 justified, 2 files (residue tranche 2026-08-11: 3 `too_many_lines` → expect, 3 cfg-dependent `unnecessary_cast` → allow-with-reason (BUG-053 — cast is real under `web_sys_unstable_apis` f64 signature, expect would be unfulfilled there), 3 stale truncation duplicates deleted; 73 raw findings fixed; a prior "completed" report was false — see History) | yes |
| examples/minwgpu/{hello_triangle,sun_grid_lines,sun_grid_lines_vulkan} | ~~(not in original census)~~ ✅ swept 2026-08-11 → 0 allows, 3 files (3 `too_many_lines` `fn run()` findings fixed by decomposition, not suppression — see History) | yes |
| module/helper/animation | ~~(not in original census)~~ ✅ swept 2026-08-11 → 3 justified (all expect); 5 stale attrs deleted, 64 latent findings fixed, `EasingBuilder::new`→`build` renamed — see History | yes |
| module/helper/scene_script | ~~(not in original census)~~ ✅ swept 2026-08-11 → 0 attrs (1 stale central-family allow deleted) | yes |
| module/helper/browser_log | ~~(not in original census)~~ ✅ swept 2026-08-11 → 0 attrs (1 stale central-family allow deleted) | yes |
| remaining 13 members with zero attr sites (behaviour_tree, browser_tools, canvas_renderer, cg_tools, cgtools, d3_scene, frame_graph, mdmath, mdmath_ai, mdmath_cg, mdmath_linalg, minwgpu, ndarray_tools) | ✅ audit-gated 2026-08-11 — 12 green as-found; canvas_renderer latent-red (11 errors) fixed properly, 0 suppressions added — see History | yes |
| examples safe tranche: minwebgl/jewelry_site, math/life, minwebgpu/{hello_triangle, hello_triangle_quickstart, deffered_rendering, renderer_pbr_scene}, minwgpu/sun_grid_lines_chunked | ~~(part of bulk row below)~~ ✅ wired + host-gate green 2026-08-11 → 2 justified expects (life's ndarray `reversed_empty_ranges` ×2), 0 allows; 4 manifests newly wired to inheritance, 3 already inherited; math_trivial's 4 findings fixed properly. Coverage caveat ~~(4 minwebgpu-dir demos wasm-target pass pending)~~ RESOLVED 4/4 2026-08-11: wasm32 `--all-features` `-D warnings` gate green for hello_triangle + hello_triangle_quickstart (`Default::default()` → `Config::default()`) and deffered_rendering (253/100 `fn run()` decomposed into 6 setup helpers + 4 per-pass recorders, following the file's own State-struct pattern); renderer_pbr_scene's wasm gate — formerly blocked by 34 gpu_hal dependency warnings — green 2026-08-11 after this task adopted and fixed gpu_hal's wasm32 lint debt post-conclusion of the concurrent lane — see History | yes |
| examples/* remainder (~~≈43~~ 23 minwebgl crates) | ~~⏸ BLOCKED~~ ✅ swept 2026-08-11 → 4 justified (filter ×2 `unnecessary_cast` + object_picking ×2 `useless_conversion`, all cfg-dependent Fix(BUG-053) allow-with-reason), 0 blanket blocks existed (the ~1000-site estimate was stale — every non-minwebgl example was already covered by the rows above); 23 manifests wired to `[lints] workspace = true`; ~152 findings fixed properly incl. 6 `fn run()` too_many_lines decompositions; wasm32 `-D warnings` gate green — see History | yes |

**New this session:** `getrandom`/`rand` wasm32 version-split gap discovered while verifying the
`mingl`/`minwebgpu` domino work (both crates' wasm32 `--all-targets` — as opposed to `--lib` —
builds fail on an unrelated, pre-existing `getrandom 0.2` vs `0.3` resolution conflict). Filed as
[BUG-079](../bug/completed/079_getrandom_wasm32_backend_version_split.md); **FIXED + closed
2026-08-11** — the renderer-established target-gated `getrandom = { version = "0.2", features =
[ "js" ] }` dev-dependency shim applied to the 5 probe-confirmed crates (mingl, minwebgl,
minwebgpu, browser_log, browser_input); all 5 now pass wasm32 `--all-features --all-targets`
`-D warnings` (probe `-0259`, verify `-0261`) — see History.

**Examples tranche (template question RESOLVED 2026-08-11):** example crates carry near-identical blanket
blocks (`implicit_return`, `min_ident_chars`, `std_instead_of_core`, ...) — a copy-pasted template.
Several of those lints are already centrally allowed-with-justification in `[workspace.lints.clippy]`
(Cargo.toml lines 71-98), so for inheriting examples the file-level copies are pure redundancy; for
non-inheriting ones the decision is template-level (adopt inheritance + delete the blocks), not
per-site. **Decision (safe-tranche increment, see History):** adopt `[lints] workspace = true`
per-crate, delete/convert file-level blocks, fix findings properly; the once-contemplated relaxed
doc-class layer for demo code proved unnecessary in practice (doc lints fired once, both hits
properly fixable) and was not created — add it only if a future crate shows genuinely-busywork doc
findings. Remainder is mechanical once minwebgl/minwebgpu land.

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
  outside this per-crate sweep's scope. See `../bug/completed/079_getrandom_wasm32_backend_version_split.md`
  (path updated on closure; fixed in a later increment below).
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
- **[2026-08-11]** `INCREMENT` — **Suppression-policy package executed (user-approved, 3 layers);
  renderer closed; tiles_tools re-greened.** Layer 1: root `Cargo.toml` centrally allows the noisy
  families with reason comments (`missing_inline_in_public_items`, `exhaustive_enums`/`structs`, casts
  ×4); `missing_errors_doc`/`missing_panics_doc` stay WARNED per user choice — renderer's gap closed by
  writing ~52 real `# Errors`/`# Panics` sections plus 12 item doc lines (gbuffer) instead of
  suppressing. Layer 2: `allow_attributes_without_reason = "warn"` ratchet — every surviving
  suppression must carry `reason = "..."`; consequence (already surfaced): workspace-wide `-D warnings`
  clippy stays red until every crate is swept, and per-crate gates need `--no-deps` (procedure step 3
  updated above). Layer 3: surviving scoped attrs converted to `#[ expect( lint, reason ) ]`;
  `#[ allow( lint, reason ) ]` only for the quirk cases (procedure step 4 updated above).
  **renderer:** crate policy block deleted from `lib.rs`; 33 scoped attrs kept (26 expect + 7 allow:
  `unused_mut` ×3 + `unused_variables` ×2 macro-body, `dead_code` ×2 RAII keepalive); 2 doc blocks
  initially misplaced by the insertion script's rsplit-anchor bug (`material/mod.rs` `fn upload`
  parameter list, `color_grading.rs` `pub fn new` body) — relocated; gate GREEN
  (`module/helper/renderer/-0004_longrun.log`, 5s), tests 82/0 across 14 targets (`renderer/-0005`).
  Census 87 → 33. **tiles_tools:** committed state gated RED under the new regime — 25 lib errors plus
  a test/bench tail (its 2026-08-10 sweep predates the regime; RED-at-pickup cause not fully
  reconstructed — the failing files were last touched by pre-sweep commits yet the crate previously
  gated green; recorded as observed facts only). Real fixes: 23 `format_push_string` →
  `write!`/`writeln!` in `debug.rs` (`use std::fmt::Write as _;`; the 18 trailing-`\n` sites then hit
  sibling `write_with_newline`, converted to `writeln!`), `#[ must_use ]` ×2 (`field_of_view.rs`),
  bench `clippy --fix` ×14 (semicolons ×6 + `explicit_iter_loop` ×8), similar-binding renames ×8
  (origin/target/center semantics, `coordinate_benchmarks.rs`), `match_same_arms` merge ×1
  (pathfinding terrain), stale cast attrs ×2 deleted (family central now). Suppressions converted:
  `unused_self` ×3 + `similar_names` ×1 → expect; `dead_code` ×3 → allow-with-reason (rustc #114557).
  New quirk discovered: rustc ignores lint attrs on macro invocations (`unused attribute` error, both
  `expect` and `allow`) — criterion_group!'s generated undocumentable `pub fn` takes a file-level
  `#![ expect( missing_docs, reason ) ]` instead (×2 bench files; fulfills correctly since the bench
  fns themselves are private). `float_cmp` ×17 across 4 test files → fn-level expect ×9
  (exact-literal round-trip assertions; epsilon rewrites would weaken the contracts under test).
  Gate ledger (`module/helper/tiles_tools/`): `-0001` 25 errors, `-0002` 16, `-0003` float_cmp,
  `-0004` warn-mode inventory (37 own warnings), `-0005` `--fix`, `-0006`/`-0007` macro-invocation
  attr dead ends, `-0008` gate GREEN, `-0009` tests 285/0 across 10 targets, `-0010` gate re-GREEN covering a post-gate cosmetic newline edit to the two bench files (`cargo test` does not compile bench targets, so the gate was re-run). Census 37 → 18.
  minwebgl NOT started in this session — the Census marks it owned by an in-flight background agent.

- **[2026-08-11]** `INCREMENT` — **gpu_hal swept: 24 → 6 justified; line_tools swept: 151 latent
  errors → 4 justified.** **gpu_hal** (census's `28` was stale; recount at pickup found 24): deleted
  the 3 inner central-family attrs from `lib.rs` and a `cast_precision_loss` attr in `device.rs`
  (families central now). Converting the remaining comment-justified allows to `expect` immediately
  exposed **14 stale suppressions** via `unfulfilled_lint_expectations` — all 13 shape-A
  `unnecessary_wraps` in `pass.rs`/`resource.rs`/`device.rs` plus 1 `needless_pass_by_value` never
  fire anymore; deleted all 14 (kept their `cfg` lines). The 6 survivors (all `device.rs`) are
  combo-dependent: a 3-probe experiment (flip to expect, gate webgpu-only / webgl-only /
  native-only, flip back) proved `unnecessary_wraps` fires only in single-backend builds where the
  surviving arm is infallible — expect would be unfulfilled under `--all-features`, so they stay
  `allow`-with-reason stating exactly that. Probes doubled as proof the 14 deletions are clean under
  all 4 feature combos. Gate ledger (`module/helper/gpu_hal/`): `-0001` inventory, `-0002` 14
  unfulfilled expectations, `-0003`..`-0006` probes, `-0007` gate GREEN, `-0008` tests 2/0 across 3
  targets. Census 24 → 6 (0 expect + 6 allow). **line_tools** (census's `32` was stale in the other
  direction: the crate has **zero** suppression attrs — it was simply never swept, and gating it
  surfaced 151 latent lib errors): `clippy --fix` cleared the mechanical tier (`-0002`); hand patch
  added ~24 real `# Errors`/`# Panics` doc sections (d2/d3 `line.rs`, `lib.rs`, `mesh.rs`,
  `program.rs`, `uniform.rs`), renamed `b_program`/`bt_program` → `body_program`/
  `body_terminal_program` (similar_names), replaced 2 wildcard match arms with explicit
  `Cap::Butt` arms, joined the split `else\nif` in `upload_with_cache` (suspicious_else_formatting)
  and added its `else_if_without_else` expect (mirror.rs precedent), added the tests crate doc, and
  converted the two existing float_cmp intent comments (`dash.rs`, `distance.rs`) to file-level
  `#![ expect( clippy::float_cmp, reason ) ]`. Survivors: 4 (`too_many_lines` on the 171-line
  `mesh_update` GL resync, `else_if_without_else`, `float_cmp` ×2) — all expect, zero allow.
  Gate ledger (`module/helper/line_tools/`): `-0001` 151 errors, `-0002` `--fix`, `-0003` warn-mode
  inventory (0 own warnings), `-0004` gate GREEN, `-0005` tests 88/0. Census 32 → 4 actual
  (0 → 4 attrs). minwebgl/minwebgpu still untouched (in-flight background agents);
  primitive_generation/tilemap_scene still blocked on minwebgl landing.

- **[2026-08-11]** `INCREMENT` — **tail-crate recount executed; animation, scene_script, browser_log
  swept (the census's "top offenders" list is now fully dispositioned).** A workspace grep recount
  surfaced 3 crates with attr sites never listed in the census. **scene_script** (1 attr) and
  **browser_log** (1 attr): both were single stale central-family allows (`cast_possible_truncation`,
  `exhaustive_structs`) — deleted (rationale comments kept), gates GREEN first try, zero attrs
  remain. Ledgers: `module/helper/scene_script/` `-0002` gate, `-0003` tests 11/0;
  `module/helper/browser_log/` `-0002` gate, `-0003` tests 15/0. **animation** (5 attrs) was
  line_tools-pattern latent-red: 4 stale central-family cast allows deleted, and the 5th — macro-body
  `new_ret_no_self` in `impl_easing_function!` — proved ALWAYS stale via expect-flip (clippy skips
  trait impls; 24 unfulfilled expectations, one per expansion), reverted to a plain rationale
  comment. The real `new_ret_no_self` site is the `EasingBuilder` trait declaration itself
  (`fn new() -> Box< T >`, `easing/base.rs:37`) — justified with expect; the honest fix is renaming
  the method to `build()`, but that is a public-API rename with ~92 call sites across animation,
  renderer, scene_script, and 2 examples — flagged for user review, not autonomous. Latent errors
  fixed: `must_use` ×19 + `let_and_return` ×1 via `--fix`, `#[ must_use ]` on `Animatable::interpolate`
  by hand (fix skips trait decls), `float_cmp` ×43 across 3 test files → file-level
  `#![ expect ]` ×3 (deterministic-arithmetic assertions, line_tools precedent). Ledger
  (`module/helper/animation/`): `-0005` 45 errors, `-0006` `--fix`, `-0007` warn inventory,
  `-0008` gate GREEN, `-0009` tests 32/0 across 5 targets. The new `#[ must_use ]` on
  `Animatable::interpolate` propagates to downstream callers, so renderer (whose green gate predated
  the edit) was re-gated: GREEN (`module/helper/renderer/-0006_longrun.log`, 49s); scene_script's
  own gate already ran after the edit. Census: animation 5 → 4 (all expect),
  scene_script 1 → 0, browser_log 1 → 0. Remaining unswept: examples tranche (~1000 sites, needs
  the template-level user decision), minwebgl/minwebgpu (in-flight background agents),
  primitive_generation/tilemap_scene (blocked on minwebgl).

- **[2026-08-11]** `INCREMENT` — **`EasingBuilder::new` renamed to `build` (user-approved proper fix,
  replacing the expect).** The trait's constructor-named method returning `Box< T >` was the one real
  `new_ret_no_self` site; user directed "only proper fixes, no workarounds. apply". Renamed: trait
  declaration + `Linear` impl (`easing/base.rs`), macro impl in `impl_easing_function!` (`lib.rs`,
  comment updated, expect deleted), 94 call sites (37 easing_test — 2 lines carry 2 calls each, which
  is why the earlier per-line grep said 35 — 12 sequencer_test, 11 interpolation_test, 2
  scene_script `tween_binding.rs`, 21 renderer tests ×4 files, 3 renderer `gltf.rs`, 1
  pingpong_animation, 7 character_control), plus 4 readme.md sites (readme is included as crate docs,
  so its blocks are the doc-tests). Inherent constructors with args (`CubicBezier::new( [..] )`,
  `Sequencer::new()`, etc.) untouched — the empty-parens + easing-type-receiver regex is the
  discriminator. Verification: combined gate GREEN for animation + scene_script + renderer
  (`module/helper/animation/-0010_longrun.log`, 32s), tests 130/0 across 23 targets (`-0011`),
  `pingpong_animation` example checks green (`examples/scene_script/pingpong_animation/-0001`).
  `character_control` (minwebgl-dependent) has the rename applied but compile verification deferred
  until `minwebgl` lands. animation census 4 → 3 (the `new_ret_no_self` expect is gone).

- **[2026-08-11]** `INCREMENT` — **latent-red audit of the 13 never-gated members: 12 green, 1
  latent-red found and properly fixed (canvas_renderer).** Diffing `cargo metadata` members against
  the census's ✅ rows left 13 crates with zero suppression attrs but no gate on record —
  behaviour_tree, browser_tools, canvas_renderer, cg_tools, cgtools, d3_scene, frame_graph, mdmath,
  mdmath_ai, mdmath_cg, mdmath_linalg, minwgpu, ndarray_tools (the line_tools lesson: zero attrs ≠
  green). One combined 13-package strict gate surfaced 11 errors, all in `canvas_renderer/src/
  renderer.rs`: `--fix` cleared needless_borrow ×5, explicit_iter_loop ×1, must_use ×1; 3 real doc
  sections written by hand (`new` — `# Returns` converted to `# Errors`; `upload_node` — `# Panics`
  for the `worldMatrix` unwrap; `render` — `# Errors` + `# Panics` covering the uniform-location and
  traversal unwraps). Re-run batch gate GREEN, tests 309/0 across 35 targets. Zero suppressions
  added. Ledger: `module/-0001` (batch, 11 errors), `module/helper/canvas_renderer/-0006` (`--fix`),
  `module/-0002` (batch gate GREEN), `module/-0003` (tests). Every module-tree member is now either
  ✅ swept, ⏳ agent-owned (minwebgl, minwebgpu), or ⏸ blocked on minwebgl (primitive_generation,
  tilemap_scene).

- **[2026-08-11]** `CORRECTION` — **tilemap_renderer's "clippy exit 0" claim doesn't hold under
  `-D warnings` — 40 sites need `reason=`, reproduces at plain default features, no adapter feature
  involved.** Investigating `adapters/webgl.rs`'s own excluded-from-sweep debt (below) surfaced this
  independently: `cargo clippy -p tilemap_renderer --lib --no-deps -- -D warnings` (default features,
  no special flags) fails with exactly 40 errors, all `allow_attributes_without_reason` — 39×
  `clippy::exhaustive_structs` + 1× `clippy::match_same_arms`, distributed `commands.rs` ×28
  (lines include 35, 53, 81, 89, 97, 115, +22 more), `assets.rs` ×9, `types.rs` ×2 (lines 99, 173),
  `backend.rs` ×1. These are 4 of the 9 files this task's own row above marked ✅ swept/46-justified.
  Root-cause investigation ruled out feature-gating as the explanation before landing on the real one:
  `lib.rs`'s `mod_interface!` block gating `commands`/`assets`/`types`/`backend` sits behind
  `#[cfg(feature = "enabled")]`, and `enabled` is this crate's own `default` feature (`Cargo.toml`
  line 15) — so all 4 files compile unconditionally, confirmed via a direct grep finding zero
  `adapter-*` cfg references anywhere in them. The 40 findings are not new and not feature-triggered;
  they were already present when task 058 marked this row ✅ — "46 justified" evidently meant the
  allow sites were individually reviewed and judged appropriate (design-level justification, e.g. the
  `RenderConfig`/`Clear` `exhaustive_structs` spot-check already on record in the row above), not that
  each one's source attribute also carries the `reason = "..."` string clippy's own
  `allow_attributes_without_reason` lint requires — the two are different bars, and only the first was
  met. (First-pass check of this session used a grep pattern matching the literal string
  `allow_attributes_without_reason`, which doesn't appear in clippy's own message text — that
  produced a false "zero hits" reading before the correct pattern, `attribute without specifying a
  reason`, surfaced the true count. Documented as a reminder against Stale Evidence Trust on a
  self-authored grep.) **Not fixed** — no `src/` edit made, consistent with this session's
  task/-only edit scope; the 40 sites need per-site `reason=` text, the same editorial judgment call
  as the original sweep's spot-checks, not a mechanical patch.
- **[2026-08-11]** `NOTE` — **`adapters/webgl.rs`'s own excluded debt, now characterized: 20
  findings under `--features adapter-webgl --no-deps -- -D warnings`.** This file was always known-
  and-deliberately out of the original 9-file/46-justified sweep (row above: "confirmed webgl.rs
  correctly untouched"); this pickup fully catalogs what that leaves outstanding, so it's tracked
  rather than just noted as excluded. Breakdown: 3 wildcard imports (lines 35-37); 3
  `too_many_arguments` functions — `fn draw` line 74 (9/7 args), a second `fn draw` overload line 153
  (10/7 args), `fn upload_image_from_path` line 1266 (8/7 args); 12 by-ref-instead-of-by-value `Copy`
  params across lines 74×2, 90, 161, 192, 360, 381, 448, 605, 718×2, 750; 1 `too_many_lines` function
  at line 923 (143/100 lines); 1 `allow_attributes_without_reason` at line 1158
  (`#[ allow( clippy::match_same_arms ) ]`, the same gap as the CORRECTION above, one site). All 20
  reproduced fresh this session (`cargo clippy -p tilemap_renderer --lib --features adapter-webgl
  --no-deps -- -D warnings`, exit 101, 60 total errors = 20 webgl.rs + 40 default-feature sites from
  the CORRECTION above). **Not fixed** — same task/-only edit scope rationale as above.
- **[2026-08-11]** `INCREMENT` — **Phase C closed: all 6 renderer `too_many_lines` expects eliminated
  by real decomposition — zero remain in the crate (`grep -rn too_many_lines src/` exit 1); 27 scoped
  attrs left (20 expect + 7 allow).** Per the standing "only proper fixes, no workarounds" directive,
  each oversized fn was split into named helpers and the suppression deleted, behavior preserved:
  **(1) `webgl/skeleton.rs` `upload`** → `displacements_update( &mut self, gl ) -> bool` (early-exit
  semantics kept: `false` = size-guard abandoned, `need_update_displacement` stays set for retry) +
  `uniforms_upload`; **(2) `webgl/animation/transition.rs` `set`** → `vector_channel`/`quat_channel`
  samplers + generic `blend< V : Animatable >`; **(3+4) `webgl/renderer.rs`**
  `FramebufferContext::new` → free `renderbuffer_multisample_create`/`texture_2d_create` (×9 call
  sites), and `render` → `nodes_collect` + `primitive_register`; **(5) `webgpu/renderer.rs`
  `WebGpuRenderer::new`** → `frame_targets_create` (hdr+depth views), `material_defaults_create`
  (1x1 dummy + sampler), `opaque_pipeline_create`/`tonemap_pipeline_create` (each owning its shader
  compile; `Device` added to the gpu_hal import list); **(6) `webgl/loaders/gltf.rs` `load`**
  (~677 lines → ~65-line dependency-ordered chain) → 13 helpers (`buffers_load`, `texture_upload`,
  `images_upload`, `gl_buffers_upload`, `textures_create`, `materials_create`, `attribute_info_make`,
  `geometry_attributes_add`, `primitive_geometry_create`, `primitive_material_resolve`,
  `meshes_create`, `nodes_create`, `skeletons_attach`, `scenes_create`) + `RiggedNode< '_ >` alias +
  `NodesCreated` struct. The strict gate itself vetted the new API shapes: round 1 flagged
  `needless_pass_by_value` (`src : Rc< str >` cloned, never consumed → now `&Rc< str >`) and
  `type_complexity` (3-tuple return → named `NodesCreated` struct) — both fixed properly, no
  suppressions added anywhere in Phase C. One E0597 surfaced when the material-variation `if let`
  became a tail expression (scrutinee temporary outliving `gltf_material`) — restructured to an owned
  `variation` binding via `.cloned()`, semantics identical (`Option::< &T >::cloned` ≡
  `.map( Clone::clone )`). 4 dead commented-out lines deleted in passing (`//let images`,
  `//gl.pixel_storei` ×2, `//a =>` match arm). Ledger: `renderer/-0009` (E0597 red), `-0010` check
  exit 0, `-0011` gate red (2 design findings), `-0012` gate exit 0, `-0013` tests 82/0 across 14
  targets, `-0014` final gate exit 0 after dead-code cleanup. Scope note: `line_tools`'s
  `mesh_update` `too_many_lines` expect (d2/line.rs:216) is NOT part of this item — the accepted
  review item named only the 6 renderer fns; that expect stays justified on record.
- **[2026-08-11]** `INCREMENT` — **Phase D opened: examples safe tranche wired + host-gated green
  (7 crates) under the adopted template decision — `[lints] workspace = true` inheritance + proper
  fixes, no relaxation layer.** Safe set computed via `cargo tree -i` inverse-dependency sweeps
  (`-minwebgl_dependents.txt` 77, `-minwebgpu_dependents.txt` 24, vs `-workspace_members.txt` 103):
  75 members transitively compile the two agent-owned crates and stay untouchable; 10 example crates
  are safe, 3 of them (minwgpu examples row above) already swept → 7-crate tranche: `jewelry_site`,
  `math_trivial` (math/life), `minwebgpu__` (hello_triangle), `minwebgpu_hello_triangle_quickstart`,
  `minwebgpu_deffered_rendering`, `minwebgpu_renderer_pbr_scene` (its `minwebgpu` dep is
  wasm32-gated — the host-target gate never compiles it), `minwgpu_sun_grid_lines_chunked`. Wired
  `[lints] workspace = true` into the 4 manifests lacking it (jewelry_site, math/life,
  minwebgpu/hello_triangle, minwebgpu/hello_triangle_quickstart); the other 3 already inherited.
  Findings: only `math_trivial` went red (batch gate `module/-0004`, exit 101, 4 errors) — all fixed
  properly per the standing directive: crate-level `//!` doc written; `pub struct Cell` de-pubbed
  (single-file bin, the `pub` was gratuitous — kills the `missing_docs` hit at the root instead of
  documenting a non-API item); `bool as u8` → `u8::from(..)`; final `println!` args inlined. Its 2
  pre-existing reasonless `reversed_empty_ranges` allows → `#[ expect ]`-with-reason (ndarray `s![]`
  negative indices are counted from the far end; clippy evaluates the endpoints as plain integers —
  false positive, legitimate survivor). The relaxed doc-class layer contemplated in option (b)
  proved unnecessary: doc lints fired only on math_trivial and both hits were properly fixable —
  zero relaxation attrs added anywhere (YAGNI). Tranche attr census after sweep: 2 expects, 0
  allows across all 7 crates (every other `grep 'allow(\|expect('` match is `Result/Option::expect()`
  method calls, not attributes). Gate: `module/-0004` red → `module/-0005` green — all 7 crates
  freshly `Checking`-linted in `-0005`, 0 warnings, exit 0, no TIMEOUT annotation. **Coverage
  caveat (adversarial self-check catch):** the 4 minwebgpu-dir demos gate their entire `run()`
  bodies behind `#[cfg(target_arch = "wasm32")]` (their `minwebgpu` dep is equally wasm-gated), so
  the host gate linted only their host-visible shell (crate docs, stub `main`) — full-body lint
  coverage requires a wasm32-target gate that would compile agent-owned `minwebgpu` and therefore
  lands with the blocked remainder below. Fully host-covered and genuinely done: `math_trivial`,
  `minwgpu_sun_grid_lines_chunked`, and `jewelry_site` (the latter trivially — it is an empty stub
  crate: `//! Empty crate`, empty `main`; no cfg gates in either). Remainder of the examples
  tranche (~43 crates incl. the blanket-template bulk, plus the 4 wasm-gated bodies above) stays
  blocked on the minwebgl/minwebgpu background agents landing.
- **[2026-08-11]** `NOTE` — **tilemap_renderer's 40-site `reason=` backlog (CORRECTION entry above)
  is being resolved by deletion, not justification, via task 084 (`⚙️ Executing`, started
  2026-08-11 11:40:39, uncommitted) — not this task.** Attempted the CORRECTION entry's deferred
  `src/` edit per explicit user authorization this pickup; found zero `#[ allow(...) ]` attributes
  remaining in any of the 4 target files (`grep -n allow` across `commands.rs`/`assets.rs`/
  `types.rs`/`backend.rs`: no hits except an unrelated prose "allows" in a `types.rs` doc comment).
  `git diff --stat -- module/helper/tilemap_renderer/` shows those exact 4 files modified
  (`assets.rs` −9, `backend.rs` −1, `commands.rs` −28, `types.rs` −2 = 40, matching the CORRECTION
  entry's count precisely) with zero insertions on the allow lines — `git diff` confirms every
  removed line is `#[ allow( clippy::exhaustive_structs ) ]` (plus the 1 `match_same_arms` site).
  Traced to `task/executing/084_tilemap_renderer_adapter_none_backend.md` (Add `adapter-none` no-op
  Backend), currently the sole in-progress work touching these files (`adapters/mod.rs`,
  `adapters/svg.rs`, `lib.rs`, `Cargo.toml`, `roadmap.md`, `tests/svg_backend_test.rs` also dirty in
  the same diff; `adapters/webgl.rs` — this task's own separately-tracked 20-finding debt — untouched
  by 084's diff). **No `src/` edit made** — the sites task 058 was asked to annotate no longer exist
  in the working tree; adding `reason=` to already-deleted attributes is not possible, and editing
  files under another task's active, uncommitted execution risks colliding with it. Row left
  unresolved pending 084 landing: if 084's deletion survives its own verification and lands, this
  row's 40-site item converges to 0 without further action here; if 084 reverts or the deletion
  proves wrong, the CORRECTION entry's original `reason=` item re-applies unchanged. Re-check this
  row after 084 completes, not before.
- **[2026-08-11]** `INCREMENT` — **tilemap_renderer's 40-site `reason=` debt resolved by deletion,
  not annotation — all 40 were `#[ allow( clippy::exhaustive_structs ) ]`, a centrally-allowed
  family (root `Cargo.toml:80`), so the file-level copies were stale duplicates** (`commands.rs`
  ×28, `assets.rs` ×9, `types.rs` ×2, `backend.rs` ×1; count-asserted strip script, exact per-file
  counts verified before write). Gate safety: run at **default features and the
  everything-but-`adapter-webgl` feature set** — `adapter-webgl` is the only path pulling
  agent-owned `minwebgl` (verified via metadata-only `cargo tree`, 0 hits; dev-deps = bytemuck
  only) — so this crate was never actually blocked; `--all-features` was the only forbidden form.
  Unblocking the lib surfaced two latent-red layers the 40 errors had been masking: **(1)**
  `tests/none_backend_test.rs` + `tests/svg_backend_test.rs` place their `//!` crate doc AFTER
  `#![ cfg( feature = ... ) ]`, so with the feature off the cfg strips the doc with the rest of the
  crate and `missing_docs` fires on the empty test crate — doc block moved above the cfg in both
  (both suites still compile AND run with features on: none_backend + svg_backend tests all present
  in the 128/128 run). **(2)** 5 more reasonless allows in feature-gated `adapters/svg.rs`, visible
  only with `adapter-svg` on: 4 were central-family duplicates (`cast_possible_truncation` +
  `cast_sign_loss`, `cast_precision_loss`, `std_instead_of_core` ×2) — deleted, keeping the
  code-explaining comments (cast saturation semantics, the 2^24 f32 bound, `core::io` instability
  rust-lang/rust#154046) and trimming only attr-referential sentences; 1 was genuinely load-bearing
  (`collapsible_match` — collapsing into a match guard would break exhaustiveness counting, E0004)
  → converted to `#[ expect ]`-with-reason, fulfillment gate-proven. Post-state: src/ minus
  webgl.rs carries exactly 1 justified expect; tests/ carry 0. Ledger: `-0016` red (40) → `-0018`
  default-features green → `-0019` red (5 feature-gated) → `-0020` safe-features green → `-0021`
  `cargo nextest run` 128/128 at the widened feature set. `adapters/webgl.rs`'s catalogued
  20-finding backlog (incl. its 1 reasonless allow) remains excluded — reachable only under
  `--features adapter-webgl`, which compiles agent-owned `minwebgl`; lands with that bucket.
- **[2026-08-11]** `NOTE` — **Consolidation gate: all 18 agent-safe module crates green
  simultaneously** (`cargo clippy -p ×18 --no-deps --all-targets --all-features -- -D warnings`,
  `module/-0006`, exit 0, 105s, 0 warnings; log shows 18/18 fresh `Checking` lines and zero
  `minwebgl`/`minwebgpu` compile lines — isolation held). Safe set recomputed fresh from
  metadata-only `cargo tree -i` (the earlier window's scratchpad dependents lists had been
  cleaned): 103 members, 74 minwebgl dependents, 23 minwebgpu dependents → 28 safe = 10 example
  crates (all swept) + 18 module crates (behaviour_tree, browser_input, browser_log,
  browser_tools, cg_tools, cgtools, d3_scene, embroidery_tools, frame_graph, mdmath, mdmath_ai,
  mdmath_cg, mdmath_core, mdmath_linalg, mingl, minwgpu, ndarray_cg, ndarray_tools). Methodology
  caveat caught during self-check: the dependents lists use `-e normal,build`, which excludes
  dev-dependencies — a crate with a min-crate dev-dep would look safe while its `--all-targets`
  gate compiled that dep anyway; the gate log's zero min-crate compile lines is the actual proof
  for these 18, not the list. Swept-but-dependent crates (tiles_tools, renderer, line_tools,
  gpu_hal, animation, scene_script, tilemap_renderer, canvas_renderer) cannot be re-proven without
  compiling agent-owned crates — they keep their individual dated proofs until the post-landing
  full-workspace gate. Observation: `module/min/minwebgl/src/buffer.rs` carries a one-line
  uncommitted diff (`StrideTrait` dropped from an import list) — in-flight agent work, untouched.
- **[2026-08-11]** `CORRECTION` — **the `NOTE` two entries above misattributed this row's 40-site
  deletion to task 084; it was this task's own concurrent work (the `INCREMENT` immediately above),
  not 084.** Cross-checked the `INCREMENT`'s claims directly rather than taking either account on
  faith: root `Cargo.toml:80` is confirmed byte-exact `exhaustive_structs = "allow"` inside
  `[workspace.lints.clippy]`; log files `module/helper/tilemap_renderer/-0016` through `-0021`
  genuinely exist on disk (not fabricated); `task/executing/084_...md`'s own History/acceptance
  checklist cites only generic "`clippy ... -D warnings` exits 0" gates, never claims or describes
  an `exhaustive_structs`/central-allow cleanup. Root cause of the misattribution: this repository
  has **zero commit history on any of these files** (`git log --oneline -- src/commands.rs` etc.
  returns empty) — every task worked this session, including 084's still-`⚙️ Executing` no-op-
  backend work and 058's own lint sweep, is accumulating in the *same* uncommitted working tree.
  `git diff --stat` shows the union of all of it; it cannot attribute a given line's change to a
  specific task. Seeing 084 was the one `⚙️ Executing` task/ file touching this crate was not
  sufficient evidence for who made a specific edit — should have checked 084's own file for a
  matching claim before writing the `NOTE`, which is what finally resolved it. No `src/` state
  changed as a result of this correction — record-accuracy fix only, per this project's
  Content-Preserving Edit standard (append, don't silently rewrite the earlier wrong entry).
- **[2026-08-11]** `NOTE` — **tilemap_renderer claims re-proven against the post-084 tree.** Task
  084 (`adapter-none` no-op backend) completed concurrently with this task's re-sweep of the same
  crate and moved to `task/completed/` after this task's gates `-0016`…`-0021` ran; its own
  completion gates occupy ledger `-0022`…`-0027`. Because 084's final edits postdate this task's
  proofs, both sweep gates were re-run fresh on the current tree: default features (`-0028`, exit
  0) and the everything-but-`adapter-webgl` feature set (`-0029`, exit 0), and the attr census
  re-verified mechanically — still exactly 1 justified expect in src/ minus webgl.rs (svg.rs
  `collapsible_match`), 0 in tests/. Census row 73 and both History accounts (this task's
  `INCREMENT`, the earlier misattribution `NOTE`, and its `CORRECTION`) are mutually consistent as
  of this entry. Concurrent task 085 (`pingpong_animation` render-command wiring, `⚙️ Executing`)
  touches only crates already outside this task's safe set — no overlap with any swept crate.
- **[2026-08-11]** `INCREMENT` — **min-crates milestone closed; primitive_generation and
  tilemap_scene swept; webgl.rs backlog independently verified closed.** (1) `minwebgl`: the
  concurrent tranche landed (16 files, +191/−101 in crate diff); this task's independent gates
  green — host `--no-deps --all-targets --all-features -- -D warnings` (`min/minwebgl/-0001`) and
  wasm32 `--lib` (`min/minwebgl/-0002`). (2) `minwebgpu`: 3 findings fixed here (2 elidable
  lifetimes in descriptor `Default` impls; `create_async`'s missing `# Errors` doc — writing it
  exposed a real copy-paste bug: its `map_err` built `FailedToCreateRenderPipeline` inside
  compute-pipeline creation while the correct `FailedToCreateComputePipeline` variant sat unused);
  wasm32 `--lib` green (`min/minwebgpu/-0011`), host green (`-0012`); wasm32 `--all-targets`
  still BUG-079-blocked. (3) `primitive_generation`: 6 pre-unblock findings + 47 surfaced by
  `--all-features` once minwebgl landed → 0, all real fixes (ufo.rs halfx/halfy/offsetx/offsety
  renames, `BoundingBox::new` for the E0639 non_exhaustive literals, curve/contour helper
  decomposition in primitive.rs, phantom `Result` dropped from `make_buffer_attribute_info`);
  gates `helper/primitive_generation/-0005`…`-0008`, tests 5/5 (`-0009`). (4) `tilemap_scene`:
  35 findings across lib + 7 test binaries → 0 (project_to_transform dedup helper in
  compile/frame.rs, SamplerFilter/MipmapMode/WrapMode re-exported through resource.rs following
  the BlendMode precedent, file-level float_cmp expects on 3 exact-pass-through test files, 3
  documented dead_code allows in tests/common); gates `helper/tilemap_scene/-0001`…`-0007`,
  suite 169/169 (`-0008`). (5) `webgl.rs` 20-finding backlog: found already fixed by the
  concurrent tranche (crate diff 8 files, +125/−147, webgl.rs mtime 13:57); verified
  independently — `--features enabled,adapter-webgl` green (`helper/tilemap_renderer/-0030`),
  `--all-features` green (`-0031`), suite 128/128 all-features (`-0032`). Attribution caveat as
  before: uncommitted shared tree, authorship per-line not provable; the gates verify the tree,
  not the author.
- **[2026-08-11]** `NOTE` — **the `allow_attributes_without_reason` ratchet is silently inert in
  every crate pinning `rust-version` < 1.81 — which is how 13 reasonless attrs survived green
  gates.** Post-gate census greps found 7 reasonless allows in minwebgl, 6 in minwebgpu, 2 in
  primitive_generation despite `-D warnings` gates; a minimal-repro probe (scratchpad crate,
  identical lint config) proved clippy MSRV-gates the lint — `rust-version = "1.80.0"` → 0
  diagnostics, `"1.81.0"` → fires (`reason =` on lint attrs stabilized in 1.81). 16 workspace
  crates pin < 1.81 (15 × 1.75 + minwebgl 1.80), so the regime's enforcement lint never ran
  there; the same pins are already contradicted by the sweep's own artifacts (renderer: 20
  `expect` attrs under a 1.75 pin; primitive_generation: 3; minwebgl: 11 reasoned allows under
  1.80 — all 1.81+ syntax). Residue cleaned this increment (correct under every policy outcome):
  12 stale central-family duplicates deleted (minwebgl 6 — casts ×5 attrs + exhaustive_structs
  ×1; minwebgpu 6 — cast_precision_loss, cast-safety comments kept; primitive_generation
  test-file cast ×1), ufo.rs `too_many_lines` allow → expect-with-reason (fulfillment
  gate-proven), and minwebgl's `unexpected_cfgs` allow replaced by the proper fix —
  `web_sys_unstable_apis` declared via `check-cfg` on `unexpected_cfgs` in root
  `[workspace.lints.rust]`, d2.rs comment updated to match. Re-gates all green: `module/-0007`
  (minwebgl host), `-0008` (minwebgl wasm32 lib), `-0009` (minwebgpu host), `-0010` (minwebgpu
  wasm32 lib), `-0012` (primitive_generation clippy — gpu_hal transiently red mid-edit under the
  concurrent agent at first attempt `-0011`, converged within a minute), `-0013` (tests 5/5).
  **Open policy question, user's call (mirrors 081's renderer pin):** raise the 16 sub-1.81 pins
  to ≥ 1.81 (makes the pins truthful and activates the ratchet workspace-wide; 1.81 keeps 081's
  `is_multiple_of` verdict intact, which needs 1.87) — or keep the pins and strip 1.81+ attr
  syntax from those crates (regresses the approved expect regime). Recommendation: raise to 1.81.
- **[2026-08-11]** `INCREMENT` — **rename call-site verification queue item closed; pingpong
  manifest warning fixed workspace-wide.** `primitives_data_to_gltf`'s new `&[ PrimitiveData ]`
  parameter and `make_buffer_attribute_info`'s dropped phantom `Result` (both from this task's
  primitive_generation sweep) had 6 stale call sites in 4 downstream example crates — all
  updated (character_control `&[ plane ]`; lottie_surface_rendering collect-then-borrow;
  animation_surface_rendering main.rs + its local primitive_data.rs ×2 — `.unwrap()` removed and
  `set_node_transform( &node )`; curve_surface_rendering `&primitives_data`), all 4 crates
  wasm32-green in one gate (repo root `-0209`). `pingpong_animation` checks green natively
  (default + adapter-svg: `module/-0014`, `-0015`) and on wasm32 with adapter-webgl (`-0016`).
  The per-invocation cargo warning ("`default-features` is ignored for tilemap_renderer") fixed
  at the root: `[workspace.dependencies.tilemap_renderer]` now declares `default-features =
  false`, and the two default-relying consumers (tilemap_scene, hexagonal_map) declare
  `default-features = true` explicitly; resolved feature sets proven identical before/after via
  `cargo tree -f "{p} {f}"` capture — pingpong intentionally drops the inert `default` token,
  activation unchanged since every adapter feature implies `enabled`; warning count on `cargo
  metadata` stderr now 0.
- **[2026-08-11]** `INCREMENT` — **reasonless-residue tranche: 6 previously-swept crates brought to
  the machine-checked standard (101 attr edits), workspace reasonless count now 0.** A Tier 2
  adversarial recount (`grep -rnE '#!?\[ ?(expect|allow)\(' … | grep -v reason`) showed the earlier
  "justified" census figures for these crates counted comment-justified allows, which the
  MSRV-inert ratchet (see the entry above) never machine-checks — 102 sites enumerated, 1 a
  comment-text false positive (tiles_tools flowfield.rs:476). All 101 real sites resolved via one
  count-asserted python patch (every per-file count asserted before any write): **27 stale
  central-family duplicates deleted** (embroidery_tools 12, mingl 10, browser_input 3,
  ndarray_cg 2 — cast-safety comments kept) plus 1 commented-out attr line (mdmath_core
  tests/inc/mod.rs); **45 `unsafe_code` allows → expect** with a uniform SAFETY-discipline reason
  (mdmath_core vector core 37, ndarray_cg mat access 8); **16 test `clippy::float_cmp` allows →
  expect** with a uniform exact-value reason (mdmath_core 5, ndarray_cg 11); **12 judgment sites**
  — comments folded into expect reasons (browser_input keyboard ×2 + input too_many_lines;
  ndarray_cg needless_pass_by_value ×2 + op_ref; mdmath_core wrong_self_convention ×2; mingl
  obj.rs `deprecated`, long rationale comment kept) and browser_input's 3 BUG-053
  `unnecessary_cast` sites kept as **allow-with-reason** (cfg-dependent: the cast is real under
  the `web_sys_unstable_apis` f64 signature, expect would be unfulfilled there; Fix(BUG-053)
  comments kept). Gates all green in one pass, expects fulfillment-proven via
  `unfulfilled_lint_expectations` under `-D warnings`: embroidery_tools
  (`helper/embroidery_tools/-0001`), browser_input (`helper/browser_input/-0001`), mingl
  (`min/mingl/-0004`), mdmath_core clippy + suite 89/89 (`math/mdmath_core/-0078`), ndarray_cg
  clippy + suite 261/261 (`math/ndarray_cg/-0008`). Post-state per crate (census rows updated to
  match): embroidery_tools 0 attrs, browser_input 6, mingl 1, mdmath_core 44, ndarray_cg 22.
  Workspace-wide recount: 0 reasonless attribute lines remain in any `*/src|tests|benches`.
- **[2026-08-11]** `INCREMENT` — **examples tranche: the 23 remaining minwebgl demo crates adopted
  workspace lints and swept green under wasm32 `-D warnings`.** Ground truth first: the census's
  "~43 crates / ~1000 sites" estimate was stale — an inheritance recount (`grep -rL '[lints]'
  examples/*/*/Cargo.toml`) showed exactly 23 crates missing `[lints] workspace = true`, all
  `examples/minwebgl/*` demos, and their sources carried **zero** blanket allow blocks. All 23
  manifests wired (section appended at EOF per convention). Survey across the 23
  (`cargo clippy --target wasm32-unknown-unknown -p … --all-features`, root ledger `-0224`)
  found ~152 findings; `--fix` applied the MachineApplicable set (`-0225`) with its output
  restyled to house codestyle (`i32::from( x )` etc.), and the judgment residue was fixed by
  hand — highlights: 6 `fn run()` too_many_lines decompositions (attributes_matrix const-data
  lift; text_msdf `load_font_texture`; sun_grid_lines `upload_scene_styling`; diamond
  `read_geometry` + `Geometry` alias; obj_viewer `load_textures` + `build_meshes`;
  make_cube_map `generate_cube_map` extracting the whole cube-map generation pass — all real
  decompositions, no suppressions), `Default::default()` → `gl::browser::Config::default()` at
  all 23 setup sites, Rc parameters taken by reference (outline, wfc), unused async/phantom
  Result dropped with callers updated (video_as_texture, outline, wfc), shader-var/coordinate
  renames for similar_names (outline, raycaster, mapgen_tiles_rendering, wfc), and 5 missing
  crate docs written from readmes. 4 pre-existing Fix(BUG-053) sites (filter ×2
  `unnecessary_cast`, object_picking ×2 `useless_conversion`) kept as **allow-with-reason** —
  cfg-dependent: the cast/conversion is an identity only under the `web_sys_unstable_apis` f64
  signature, so expect would be unfulfilled in the default i32 build. Re-survey (`-0234`)
  showed 6 residual warnings (the 4 above + 2 introduced by the decompositions: obj_viewer's
  now-unused `t`, diamond's tuple type_complexity), all fixed; final gate `-0235` — all 23
  crates, wasm32, `-D warnings` — **0 warnings, exit 0**. Post-state: 4 justified attrs across
  the 23 (recount: 0 reasonless); every example crate in the repository now inherits
  `[workspace.lints]`.
- **[2026-08-11]** `INCREMENT` — **safe-tranche wasm coverage caveat resolved 3/4: the cfg-gated
  minwebgpu demos now gate green under their real target.** The safe-tranche row's host gates
  could not see these demos' code (fully `#[cfg(target_arch = "wasm32")]`-gated), so a wasm32
  `--all-features` `-D warnings` pass was owed. Survey (`-0239`, re-survey `-0254`) found:
  hello_triangle + hello_triangle_quickstart each 1 finding (`default_trait_access` —
  `Default::default()` → `gl::browser::Config::default()`, same fix as the minwebgl tranche);
  deffered_rendering `fn run()` at 253/100 too_many_lines — decomposed following the file's own
  State-struct pattern into 6 setup helpers (`create_texture_views` returning the
  `[ pos, albedo, normal, depth ]` view array, `create_uniform_bind_group` carrying the
  Fix(BUG-051) block, `create_gbuffer_pipelines`, `create_gbuffer_bind_group`,
  `create_lighting_pipeline`, `create_light_bindings`) + 4 per-pass recorders
  (`record_gbuffer_pass` — bundled `color_views : [ &GpuTextureView; 3 ]` to stay ≤7 params,
  `record_lighting_pass`, `record_light_vis_pass`, `record_light_update_pass`), no suppressions.
  First combined gate attempt (`-0255`) failed with exit 101 on 34 warnings from **gpu_hal** —
  discovery: trailing `-- -D warnings` also denies workspace path-dependencies compiled in the
  same invocation, and renderer_pbr_scene's dep tree pulls in gpu_hal, mid-sweep in the
  concurrent agent's lane. Split gate (`-0257`): hello_triangle + hello_triangle_quickstart +
  deffered_rendering, wasm32 `--all-features` `-D warnings` — **0 warnings, exit 0**.
  renderer_pbr_scene's wasm gate stays deferred until gpu_hal lands (census row 80 annotated).
- **[2026-08-11]** `INCREMENT` — **BUG-079 fixed + closed: wasm32 `--all-targets` builds restored
  workspace-wide.** The fix already existed in-repo as precedent: `helper/renderer/Cargo.toml`
  carries a documented target-gated shim for exactly this path (`test_tools` → rand 0.8 →
  getrandom 0.2, backend-less on wasm32 because `.cargo/config.toml`'s
  `getrandom_backend="wasm_js"` cfg only reaches the 0.3+ generation). A 6-crate probe (`-0259`)
  narrowed the affected set: tilemap_renderer clean (its dev graph never reaches rand 0.8 on
  wasm32), renderer pre-shimmed, 5 broken — mingl, minwebgl, minwebgpu, browser_log,
  browser_input. Applied the shim verbatim to all 5 (`[target.'cfg(target_arch =
  "wasm32")'.dev-dependencies] getrandom = { version = "0.2", features = [ "js" ] }`, each with
  a `Fix(BUG-079)` root-cause/pitfall comment; deliberately version-local, dev-only,
  target-gated — not `workspace = true`, the workspace getrandom is 0.3+). Verify (`-0261`): all
  5 crates `cargo clippy -p <c> --target wasm32-unknown-unknown --all-features --all-targets
  -- -D warnings` — **exit 0, zero warnings each**; these are the first-ever successful wasm32
  `--all-targets` compiles of these crates' test targets, and they surfaced no masked findings.
  BUG-079 closed (Round 0, self-accepted): report moved to
  `../bug/completed/079_getrandom_wasm32_backend_version_split.md`, both readme tables updated.
  The minwebgpu census row's `--all-targets` caveat (row 69) and the "New this session" note are
  resolved accordingly.
- **[2026-08-11]** `INCREMENT` — **all-warnings sweep closed: census gap swept, gpu_hal wasm32 lint
  debt adopted + fixed, BUG-080 + BUG-091 closed, 4-phase workspace gate green.** The user's
  standing directive ("fix all warnings / all clippy errors/warnings") exposed a tranche-boundary
  coverage gap recorded honestly here: the census rows track `#[allow]` *attribute sites*, so crates
  with zero attrs could still carry live clippy findings the attr-oriented tranches never gated.
  Sweeping the gap found 5 dirty example crates, all fixed properly (no suppressions added):
  minwebgl_gltf_viewer, area_light (2 findings, run() decomposed via `setup_gui`/`create_shaders`/
  `initial_light`/`setup_camera`/`draw_skull` helpers), animation_blending (dir
  `animation_amplitude_change/` — 8 findings: format inlining, `Config::default`, camera/parts
  extraction), postprocessing (4: pass-by-value ×2, 169-line `setup` deduped via a
  field-accessor-fn-pointer `add_grading_slider` helper ×8), pbr_lighting (25: the largest —
  matches!/collapsed if-lets, single-variant wildcards → explicit `Light::Spot` arms,
  `needless_range_loop` → `for color in colors`, dropdown callback split into per-mode `apply_*`
  handlers, position sliders deduped via `fn( &mut Settings ) -> &mut f32` accessors; pre-existing
  quirks preserved verbatim: lights double-added to `scene.children`, redundant `light_mode`
  re-assignments). Patchers `-0290`/`-0291`/`-0292` (count-asserted). **gpu_hal stale-cache phantom
  failures diagnosed:** gate runs `-0290` (74s) and `-0291` (19s) failed with 16 gpu_hal errors
  flagging `cfg( target_arch = "wasm32" )`-gated items during a HOST build — physically impossible
  fresh; isolated busted-fingerprint repro (`touch` + re-clippy) = 0 findings, and the exact Phase-1
  command then passed foreground. Root cause: stale cached diagnostics replayed from the concurrent
  lane's clippy units. Lesson: before trusting a gate failure on a crate another actor touched,
  reproduce in isolation with a busted fingerprint. The 16 findings were nonetheless REAL in their
  home config (wasm32): after confirming the concurrent lane's 086/087 arc concluded, this task
  adopted the debt — `webgl.rs` `to_i32`/`to_u32` `#[ must_use ]` ×2, patcher `-0293`: 13 more
  `#[ must_use ]` on the `as_webgl` accessor family (resource.rs ×8, device.rs ×3, pass.rs ×2 —
  completing the family symmetry; the concurrent lane had already covered `as_webgpu`/`as_native`),
  `# Errors` doc section on `new_webgl` (worded from the verified `From< WebglError > → Error::WebGl`
  mapping), and a `cfg_attr( all( webgpu, webgl, wasm32 ), expect( match_same_arms ) )` on
  `read_pixels` (arms unmergeable — variants feature-gated; plain `expect` would be unfulfilled on
  host/single-feature builds). gpu_hal clippy green on host AND wasm32 `--all-features`. This also
  unblocked `minwebgpu_renderer_pbr_scene`'s wasm32 gate (green — safe-tranche caveat now 4/4).
  **BUG-091 fixed + closed** (minwebgl `d2.rs` `as f64` ×2 → `f64::from`, live only under
  RUSTFLAGS-override configs post-BUG-053; verified via its exact MRE + branch-activating +
  normal-config clippy, all exit 0). **BUG-080 closed** (all 7 struct-literal sites now
  `BoundingBox::new`: text_rendering ×5 by this sweep, ufo.rs ×2 landed via `96bb2aef`; both
  per-crate Verify Commands exit 0). Both reports → `../bug/completed/`, both readme tables updated.
  **Final gate:** first relaunch (`-0293_longrun.log`) failed in 4s on tilemap_renderer
  `too_many_lines` (`adapters/webgl.rs:814 load_images`, 107/100) — that is task 092's live claim
  (expires 20:59) mid-flight WIP, not swept-scope residue; per the concurrent-actor protocol it was
  NOT touched. Relaunched (`-0295_longrun.log`) with the live-lane dependency cone excluded
  (tilemap_renderer, tilemap_scene, hexagonal_map, pingpong_animation): **all 4 phases green in
  476s** — Phase 1 host `--workspace --all-targets --all-features` clippy `-D warnings`; Phase 2
  wasm32 clippy over the 20 touched example packages; Phase 3 `--workspace` nextest **1220/1220**;
  Phase 4 `--workspace` doc tests. Zero `is ignored` manifest-warning hits in the gate log — the
  user's originally-reported warning class is clean. The excluded cone re-enters workspace gating
  when task 092's lane concludes (its own per-crate gates govern it meanwhile).

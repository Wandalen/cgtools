# Native test coverage for renderer's remaining glTF loader pure functions (`quat_sequence`, `weights_sequence`, `attribute_descriptor_make`, `nodes_create`, `skeletons_attach`, `scenes_create`, `skeleton_displacements_data_load`)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** module/helper/renderer
- **repo_identity:** self
- **verified_by:** self (Tier 2 Dual-Role Self-Check)
- **verification_date:** 2026-08-18
- **blocked_by:** null

## Goal

Earlier in this session, 7 previously-private pure functions across renderer's
two glTF loader files were made `pub`, exported via `mod_interface!`, and
given native (non-wasm, GPU-free) test coverage — following the same
private→pub→tested pattern already established by task 118 (`light_list_get`,
sibling loader) and task 223 (`channel_decode`/`vec3_sequence`, same
animation-loader file as 2 of these 7). This work was never given its own
task file; this task documents it retroactively.

In `module/helper/renderer/src/webgl/animation/loaders/gltf.rs`:
`quat_sequence` (lines 97-189) and `weights_sequence` (lines 322-417) — the
two sibling sequence-builders task 223 explicitly deferred as "same-shape
follow-up work" in its own Out of Scope. Both are now `pub fn` with a doc
comment and are in the file's `mod_interface! { own use { ... } }` block
(lines 528-538) alongside `channel_decode`/`vec3_sequence`/`load`.

In `module/helper/renderer/src/webgl/loaders/gltf.rs` (the sibling,
non-animation loader — task 118's own file): `attribute_descriptor_make`
(lines 851-868), `nodes_create` (lines 1173-1246), `skeletons_attach` (lines
1254-1297), `scenes_create` (lines 1303-1324), and
`skeleton_displacements_data_load` (lines 156-274) — five more pure
data-transform functions (accessor→`BufferDescriptor` computation,
node-hierarchy wiring, skin/morph attachment, scene assembly, morph-target
displacement packing), none containing a `gl`/`GL`/`WebGl` call in their own
bodies (`skeletons_attach`'s doc comment notes the one nearby GL call,
`PbrMaterial::define_add`, is only ever reached once a real `Skeleton` comes
back — outside this function's own body, confirmed by reading
`pbr.rs:459`'s definition, a pure `HashMap` insert + `Cell` flag set with no
GL-context parameter). All five now `pub fn` with doc comments, in the
file's `mod_interface!` block (lines 1436-1456).

6 of these 7 are pure visibility-only refactors — zero behavior change,
confirmed by the crate's full existing (pre-change) test suite continuing to
pass unmodified through `load()`'s own end-to-end coverage. `weights_sequence`
is the exception: its `pub`-ification was bundled with the actual fix for
BUG-262 (`task/bug/completed/262_gltf_weights_sequence_panics_on_zero_targets.md`)
— a real panic via `[T]::chunks(0)` when a glTF document's morph-weight
channel legitimately declares 0 targets. The `targets == 0` early-return
guard (lines 300-309) is genuine new behavior, not a refactor; this task's
own scope for `weights_sequence` is limited to confirming and registering
its resulting `pub`+tested+fixed state, not re-documenting BUG-262's fix
(see Related Documentation).

Success is observable as: all 7 functions `pub fn`, all exported via
`mod_interface!`, each covered by ≥1 native test across 4 test files
(`gltf_animation_loader_test.rs`, `gltf_attribute_descriptor_test.rs`,
`gltf_node_scene_test.rs`, `gltf_skeleton_displacements_test.rs`), full
crate suite green (`cargo nextest run -p renderer --all-features`) and
clippy clean (`cargo clippy -p renderer --all-targets --all-features --
-D warnings`) — both confirmed this session (186/186 tests passing, 0
clippy warnings).

## In Scope

- `module/helper/renderer/src/webgl/animation/loaders/gltf.rs` —
  `quat_sequence`/`weights_sequence` visibility (`pub fn`) + `mod_interface!`
  export
- `module/helper/renderer/src/webgl/loaders/gltf.rs` —
  `attribute_descriptor_make`/`nodes_create`/`skeletons_attach`/
  `scenes_create`/`skeleton_displacements_data_load` visibility (`pub fn`) +
  `mod_interface!` export
- `module/helper/renderer/tests/gltf_animation_loader_test.rs` — 3 tests
  (T01-T03: `weights_sequence` ×2, `quat_sequence` ×1) extending task 223's
  existing file
- `module/helper/renderer/tests/gltf_attribute_descriptor_test.rs` — new
  file, 2 tests (T04-T05)
- `module/helper/renderer/tests/gltf_node_scene_test.rs` — new file, 3 tests
  (T06-T08)
- `module/helper/renderer/tests/gltf_skeleton_displacements_test.rs` — new
  file, 2 tests (T09-T10)
- Registering this task in `task/readme.md`'s Tasks Index, and correcting
  the 4 test files' doc-comment citations from the placeholder "task 441"
  (an internal session-tracker ID, not a real `task/` entry) to this task's
  real ID

## Out of Scope

- `channel_decode`/`vec3_sequence` in the animation-loader file — task 223's
  own scope, already filed separately
  (`task/verifying/223_renderer_gltf_animation_loader_native_test.md`); not
  restated, not duplicated, not modified by this task
- `load()` (animation loader) and the sibling file's already-`pub` functions
  (`load`, `asset_uri_resolve`, `light_list_get`, `light_get`,
  `skeleton_transforms_data_load`, `material_variation_resolve` — task 118's
  own scope) — untouched
- Splitting the 4 over-50-line functions
  (`skeleton_displacements_data_load` 119, `nodes_create` 74, `quat_sequence`
  93, `weights_sequence` 96 lines) into ≤50-line pieces — unlike task 223's
  own precedent for `vec3_sequence`, this was not done when these 4 were
  exported; documented as a known deviation in Delivery Requirements below,
  not silently claimed compliant. No lint in this crate currently fires on
  function length (confirmed clean clippy at up to 119 lines), so this is a
  documentation-accuracy matter, not a build-blocking one. Splitting these
  4 is deferred as its own follow-up, not performed by this retroactive
  filing.
- Re-documenting BUG-262's fix — already fully documented in its own bug
  file (5-section test documentation + 3-field source comment); this task
  only confirms/registers the resulting `pub`+tested state
- `module/helper/renderer/tests/readme.md`'s existing Responsibility Table
  row for `gltf_animation_loader_test.rs` — its description predates this
  task's 3 added tests and does not mention `quat_sequence`/
  `weights_sequence`; left as-is, outside this task's explicit mandate (the
  other 3 new files' own rows are already accurate)

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code (code tasks) — N/A for this
    retroactive filing: the work was already complete when this task was
    filed; the Test Matrix below reconstructs the coverage actually
    delivered
-   Every Test Matrix case is backed by a test that failed before its
    implementing change landed (code tasks) — holds for T01 (BUG-262's own
    fail-before/pass-after regression test, per that bug's own file); T02-T10
    are net-new coverage of already-correct pure logic (6 of 7 functions),
    not regression-driven, since no bug existed for them
-   Minimum code to satisfy Test Matrix — no features beyond requirements
    (code tasks)
-   `cargo nextest run -p renderer --all-features` passes with zero
    failures — confirmed: 186/186 pass
-   `cargo clippy -p renderer --all-targets --all-features -- -D warnings`
    passes clean — confirmed: 0 warnings
-   Public items have `///` doc comments (code tasks) — confirmed for all 7
    functions, each stating its pure/GPU-free contract
-   **Known deviation:** 4 of the 7 functions exceed this workspace's soft
    ≤50-line-per-function convention (`skeleton_displacements_data_load`
    119, `nodes_create` 74, `quat_sequence` 93, `weights_sequence` 96
    lines). Task 223's own precedent split `vec3_sequence` on export to stay
    under this bound; these 4 were exported as-is. Recorded here honestly
    rather than claimed compliant; does not block a clean clippy pass (no
    length lint configured in this crate). Splitting is deferred, not
    performed by this task.
-   Independent verification: N/A — retroactively filed by the same actor
    who performed the work, in the same session; Tier 2 (Dual-Role
    Self-Check) is this session's MAAV verification cap (see
    `## Verification Record`)
-   Task filed directly to `task/completed/`, reflecting already-verified,
    already-complete state (no separate lifecycle transition needed)

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Morph-weight Linear channel, 2 keyframes, `targets == 0` (BUG-262 regression shape) | `weights_sequence(channel, buffers, 0)` | `None`, not a panic (`[T]::chunks(0)` guard) |
| T02 | Morph-weight Linear channel, 2 keyframes, 2 targets | `weights_sequence(channel, buffers, 2)` | `Some(Sequence)` with 2 tweens; `start_value`/`end_value` match authored weight vectors |
| T03 | Rotation Linear channel, 2 keyframes | `quat_sequence(channel, buffers)` | `Some(Sequence)` with 2 tweens; tween endpoints match authored quaternions |
| T04 | Tightly-packed `FLOAT` `VEC2` accessor, nonzero `byteOffset`, no `byteStride` | `attribute_descriptor_make(accessor)` | `DataType::F32`, `natoms == 2`, `offset` scaled from byte to element units, `stride == 0`, `normalized == false` |
| T05 | `normalized: true` `UNSIGNED_BYTE` `VEC4` accessor, zero offset | `attribute_descriptor_make(accessor)` | `DataType::U8`, `natoms == 4`, `normalized == true` |
| T06 | 3-node hierarchy: root with 2 children (one mesh+translation, one plain) | `nodes_create(gltf, meshes)` | One `Node` per glTF node; child order preserved; mesh `Rc` shared, not cloned; non-mesh node is `Object3D::Other` |
| T07 | Mesh node with no skin and no morph targets | `skeletons_attach(nodes, rigged_nodes, buffers)` | Mesh's `skeleton` stays `None` — unrigged node left untouched |
| T08 | glTF document declaring 2 scenes (one referencing a node, one empty) | `scenes_create(gltf, nodes)` | One `Scene` per glTF scene; membership matches each scene's declared node list |
| T09 | Primitive with no morph targets declared | `skeleton_displacements_data_load(None, vertex_counts, None, buffers)` | `None` — no displacement data to pack |
| T10 | Primitive with one `POSITION` morph target over 3 vertices + mesh-level weights | `skeleton_displacements_data_load(Some(targets), counts, Some(weights), buffers)` | `Some(DisplacementsData)`; packed buffer matches the expected per-vertex `[pos.xyz,1.0][normal.xyz,1.0][tangent.xyz,1.0]` layout (zero-filled normal/tangent placeholders); weights preserved |

## Acceptance Criteria

-   All 7 functions are `pub fn`, exported via `mod_interface!`, each with a
    doc comment stating their pure/GPU-free contract
-   Every Test Matrix row (T01-T10) has a corresponding passing native test
-   `weights_sequence`'s `targets == 0` guard (BUG-262) is exercised by a
    dedicated regression test, not just incidentally covered
-   `channel_decode`/`vec3_sequence` (task 223's own scope) remain untouched
    by this task
-   Full crate test suite and clippy both pass clean

## Verification

**Execution:** This section was walked via Tier 2 (Dual-Role Self-Check —
this session's MAAV verification cap; see
`governance/maav.rulebook.md § MAAV : Verification Tier Selection`): the
filer performs two distinct passes over the already-completed work — one
confirming, one adversarial — rather than dispatching an independently
verified handoff. Both passes are recorded in `## Verification Record`
below; the checklist items here are checked directly against real command
output gathered this session, not deferred to a separate hand-off stage.

### Checklist

**Function surface (src/)**
- [x] C1 — Are all 7 functions `pub fn` with a `mod_interface!` export? —
  YES: `grep -n "pub fn quat_sequence\|pub fn weights_sequence"
  module/helper/renderer/src/webgl/animation/loaders/gltf.rs` and
  `grep -n "pub fn attribute_descriptor_make\|pub fn nodes_create\|pub fn
  skeletons_attach\|pub fn scenes_create\|pub fn
  skeleton_displacements_data_load"
  module/helper/renderer/src/webgl/loaders/gltf.rs` both match; both files'
  `mod_interface! { own use { ... } }` blocks list all 7 names
- [x] C2 — Does each of the 7 carry a `///` doc comment stating a pure,
  GPU-free contract? — YES: confirmed by direct read of each function's
  preceding doc comment (animation file lines 87-96/310-321; loaders file
  lines 144-155/840-850/1168-1172/1248-1253/1299-1301)

**Test coverage**
- [x] C3 — Does `cargo nextest run -p renderer --all-features` report all
  10 new test cases passing? — YES: 186/186 total tests pass (includes the
  10 new plus task 223's own 4 plus the crate's pre-existing suite)
- [x] C4 — Does the BUG-262 regression test (T01) assert `None`, not just
  absence of a panic via an unwrapped call? — YES:
  `weights_sequence_returns_none_instead_of_panicking_when_targets_is_zero`
  asserts `.is_none()` directly on the returned `Option`
- [x] C5 — Do T02/T03/T06-T10 assert real field values (not just
  `is_some()`)? — YES: confirmed by direct read of each test body;
  e.g. T02 asserts `tweens[0].start_value == first_value` /
  `tweens[1].end_value == second_value` against authored fixture vectors;
  T10 asserts the full 24-element packed `f32` buffer against an explicit
  expected layout

**Code quality**
- [x] C6 — Does `cargo clippy -p renderer --all-targets --all-features --
  -D warnings` pass clean? — YES: 0 warnings (longrun-wrapped run, exit 0)
- [x] C7 — Are the 4 test files' doc comments accurate about which
  functions they cover? — YES, after Step 4 of this task's own execution
  (see History) corrected the stale "task 441" citation in each

**Out of Scope confirmation**
- [x] C8 — Is `channel_decode`/`vec3_sequence` untouched by this task? —
  YES: both remain exactly as task 223 left them; no edits made to their
  bodies or signatures by this filing
- [x] C9 — Does `weights_sequence`'s dual nature (pub export bundled with
  BUG-262's actual fix) get stated accurately rather than characterized as
  a pure refactor? — YES: see Goal section above and Related Documentation

### Measurements

- [x] M1 — `cargo nextest run -p renderer --all-features` → 186 tests run,
  186 passed, 0 skipped (confirmed via longrun-wrapped run this session)
- [x] M2 — Longest of the 7 newly-exported functions:
  `skeleton_displacements_data_load` at 119 lines (recorded, not hidden —
  see Delivery Requirements' Known Deviation)

### Invariants

- [x] I1 — compiler clean: `cargo check -p renderer --all-features` → 0
  errors (implied by the clean clippy + nextest runs, both of which compile
  the crate first)
- [x] I2 — `channel_decode`/`vec3_sequence` (task 223's scope) remain
  exactly as before — confirmed by their line ranges/bodies being outside
  every diff this task's own work touched

### Anti-faking checks

- [x] AF1 — T01's assertion checks `.is_none()` on the actual return value,
  not merely that the call completes without panicking, which would pass
  even under a weaker "catch and ignore" fix — confirmed by direct read of
  the test body
- [x] AF2 — T02/T03/T10's assertions check specific numeric field values
  against fixture-authored data, not just `Some(_)` — confirmed by direct
  read of each test body (see C5 above)

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | Adversarial pass questioned whether documenting BUG-262 (a separate, already-completed bug) inside this task's Goal/Related Documentation mixes two units of work | Confirmed non-blocking: BUG-262 is only cross-referenced for accuracy (its fix explains `weights_sequence`'s dual nature), never restated or re-verified here; single crate (`renderer`), matching task 223's own precedent exactly |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 1 issue, resolved | — |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` — Task filed retroactively by
  user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/, documenting
  already-completed work: 7 previously-private pure glTF-loader functions
  (`quat_sequence`, `weights_sequence`, `attribute_descriptor_make`,
  `nodes_create`, `skeletons_attach`, `scenes_create`,
  `skeleton_displacements_data_load`) made `pub`, exported via
  `mod_interface!`, and given native test coverage — closing task 223's own
  deferred `quat_sequence`/`weights_sequence` follow-up and extending task
  118's precedent to the remaining 5 functions in the sibling loader file.
  The 4 new/extended test files' doc comments previously cited a
  placeholder "task 441" (an internal session-tracker ID, not a real
  `task/` entry); corrected to cite this task's real ID as part of this
  same filing.

## Related Documentation

- `task/verifying/223_renderer_gltf_animation_loader_native_test.md` — the
  precedent this task follows exactly for `quat_sequence`/`weights_sequence`
  (same file, same "pure GPU-free" framing); explicitly named these two as
  its own deferred follow-up, which this task closes
- `task/accepting/118_renderer_gltf_light_extension_parsing_test.md` — the
  precedent this task follows for the sibling loader file's export shape
  (`light_list_get`)
- `task/bug/completed/262_gltf_weights_sequence_panics_on_zero_targets.md`
  — the bug fix that independently drove `weights_sequence`'s
  `pub`-ification (the `targets == 0` guard); this task's own scope for
  `weights_sequence` is limited to confirming/registering its resulting
  `pub`+tested state, not re-documenting the fix itself
- `module/helper/renderer/tests/gltf_light_parsing_test.rs` — the inline-
  JSON-fixture-via-`Gltf::from_slice` pattern all 4 new/extended test files
  reuse

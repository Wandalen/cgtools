# Native test coverage for glTF KHR_lights_punctual light-extension parsing

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🎯 (Verified)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/renderer
- **verified_by:** user1@w002
- **verification_date:** 2026-08-16
- **blocked_by:** null

## Goal

Give `renderer`'s glTF loader's light-extraction logic (`light_list_get`,
`src/webgl/loaders/gltf.rs:291`, currently private) native, GL-context-free
test coverage — promoting it to `pub` and exporting it via `mod_interface!`'s
`own use` list, the exact same promotion `asset_uri_resolve` already went
through for the loader's URI-resolution logic (`tests/gltf_loader_tests.rs`
is the existing precedent this task mirrors). Matters now because
`docs/layer/005_l4_scene_model.md` states glTF's `load()` "requires a live
GPU context just to parse" and is "not off-GPU-validatable" (lines 23, 31) as
a blanket claim over the whole loader — but `light_list_get` is a concrete
counter-example: it takes an already-parsed `&gltf::Gltf` document and
produces `Light`/`PointLight`/`DirectLight`/`SpotLight` domain values through
pure pattern-matching and field mapping, with zero
`WebGl2RenderingContext`/`gl::` calls anywhere in its body (confirmed by
reading the function in full). This is gap #15 from the 2026-08-15/16
docs/layer gap audit. Bounded to a 2-line visibility/export change plus one
new native test file in this one crate. Testable: `cargo test -p renderer
--test gltf_light_parsing_test` exits 0 with all 4 Test Matrix cases passing.

## In Scope

- `module/helper/renderer/src/webgl/loaders/gltf.rs`: change `fn
  light_list_get( gltf : &gltf::Gltf ) -> Option< FxHashMap< usize, Light > >`
  (line 291) to `pub fn light_list_get( ... )`; add `light_list_get` to the
  existing `crate::mod_interface! { own use { GLTF, load, asset_uri_resolve
  }; }` list (→ `own use { GLTF, load, asset_uri_resolve, light_list_get };`).
- New `module/helper/renderer/tests/gltf_light_parsing_test.rs` (native, no
  feature gate needed — `gltf` is pulled in by the crate's own default-on
  `enabled` feature, `Cargo.toml` line 62, with the `KHR_lights_punctual`
  cargo feature already turned on), feeding hand-authored minimal glTF JSON
  strings through `gltf::Gltf::from_slice` then `light_list_get`, covering: a
  mixed fixture with one Point (with `range`), one Directional, and one Spot
  (with nested `spot` object) light at increasing indices; a Point light
  missing `range` (must be silently skipped, not inserted, per the function's
  own `continue` branch); an empty `lights: []` array under a present
  extension (must return `Some` with an empty map); and no
  `KHR_lights_punctual` extension key at all (must return `None`).

## Out of Scope

- `light_get` (`gltf.rs:358`) — the sibling function pairing a glTF node to
  its light instance; needs a `&Node` parameter this task's fixtures don't
  construct, a separate and larger surface than the light-list extraction
  this task targets.
- Any GL-context-dependent part of the loader (`load`, `texture_upload`,
  `meshes_create`, `skeleton_load`, etc.) — out of reach for native tests,
  the same accepted gap tasks 114/115 already carved out for their own
  crates (no native/offscreen WebGL2 provider exists in this workspace).
- Changing `Light`/`PointLight`/`DirectLight`/`SpotLight` type definitions —
  already `pub` and already exported via `src/webgl/light.rs`'s own
  `mod_interface!`; this task only exercises them as read-only expected
  values.
- `module/helper/renderer/src/webgl/animation/loaders/gltf.rs` — the
  separate animation-specific glTF ingestion module cited in
  `docs/layer/005_l4_scene_model.md` line 49; a distinct loader file, not
  touched by this task.
- Asserting `position`/`direction` field values — both are always
  `F32x3::default()` regardless of fixture input (never parsed from JSON by
  this function), so asserting them would pin `Default`'s behavior, not
  `light_list_get`'s own logic.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Visibility/export change lands with zero behavior change to any
    currently-passing test — `light_list_get`'s body is untouched, only its
    visibility and export status change
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its
    implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   `verb/test` passes with zero failures and zero warnings
-   No function exceeds 50 lines; no duplication; public items have `///`
    doc comments
-   Independent verification passes per `§ Acceptance Verification :
    Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to
    `task/completed/`

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Mixed fixture: index 0 Point (`range:10.0, color:[1,0,0], intensity:500.0`), index 1 Directional (`color:[0,1,0], intensity:2.5`), index 2 Spot (`color:[0,0,1], intensity:800.0, spot:{innerConeAngle:0.1, outerConeAngle:0.5}`, no `range`) | `light_list_get` | Returns `Some` map with exactly 3 entries: index 0 is `Light::Point` (`color=[1,0,0]`, `strength=500.0`, `range=10.0`); index 1 is `Light::Direct` (`color=[0,1,0]`, `strength=2.5`); index 2 is `Light::Spot` (`color=[0,0,1]`, `strength=800.0`, `range=10.0` via `unwrap_or` default, `inner_cone_angle=0.1`, `outer_cone_angle=0.5`) |
| T02 | Fixture with 1 Point light, `range` field entirely absent | `light_list_get` | Returns `Some` with an **empty** map — the light is silently skipped (`continue`), never inserted |
| T03 | Fixture with `KHR_lights_punctual` extension present but `lights: []` | `light_list_get` | Returns `Some` with an empty map — distinct from T04's `None` |
| T04 | Fixture with no `KHR_lights_punctual` extension key at all | `light_list_get` | Returns `None` |

## Acceptance Criteria

-   `light_list_get` is declared `pub fn` in `src/webgl/loaders/gltf.rs`
-   `light_list_get` is present in the `mod_interface!` `own use { ... }`
    list, reachable as `renderer::webgl::loaders::gltf::light_list_get`
-   `tests/gltf_light_parsing_test.rs` exists with all 4 Test Matrix cases
    passing
-   No pre-existing test in `renderer`'s suite regresses
-   Every Test Matrix row has a corresponding passing test

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting (via EXEC_COMPLETE, ⚙️ → 📦, followed by an acceptance CLAIM, 📦 → 🔎).

### Checklist

Desired answer for every question is YES.

**Export**
- [ ] C1 — Is `light_list_get` declared `pub fn` in `src/webgl/loaders/gltf.rs`?
- [ ] C2 — Is `light_list_get` present in the `mod_interface!` `own use { ... }` list alongside `GLTF, load, asset_uri_resolve`?

**Tests**
- [ ] C3 — Does `tests/gltf_light_parsing_test.rs` exist with 4 tests, one per Test Matrix row?
- [ ] C4 — Does T01's assertion check all 3 light kinds' field values (color/strength/range/cone-angles), not just variant discriminant or map length?

**Out of Scope confirmation**
- [ ] C5 — Is `light_get` (the sibling node-pairing function) left unmodified and unexported?
- [ ] C6 — Do `Light`/`PointLight`/`DirectLight`/`SpotLight` type definitions in `src/webgl/light.rs` remain unmodified (`git diff` shows no edits there)?
- [ ] C7 — Does `src/webgl/animation/loaders/gltf.rs` remain unmodified (`git diff` shows no edits under that path)?
- [ ] C8 — Does the new test file avoid asserting on `position`/`direction` field values?
- [ ] C9 — Does `gltf.rs`'s diff touch only `light_list_get`'s signature (`fn` → `pub fn`) and the `mod_interface!` export list — no GL-context-dependent function body (`load`, `texture_upload`, `meshes_create`, `skeleton_load`) modified?

### Measurements

- [ ] M1 — new test count: `cargo test -p renderer --test gltf_light_parsing_test 2>&1 | grep -c "test result: ok"` → 1 (was: file did not exist)

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check -p renderer --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — T01's 3 light-kind assertions use 3 distinct expected `Light` variant constructions, not the same expected value asserted 3 times — checked by reading the literal expected values in the test file, not merely by the test passing
- [ ] AF2 — T02's fixture genuinely omits the `range` key entirely (not merely sets it to `null` or `0.0`) — checked by reading the literal JSON string in the test file

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | — | — |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-16]** `FILED` — Task filed via `/doc_tsk` Phase 3 (docs/layer gap audit): add native test coverage for glTF `KHR_lights_punctual` light-extension parsing to `renderer`.

## Related Documentation

- `docs/layer/005_l4_scene_model.md` — the layer doc whose blanket "requires a live GPU context just to parse" claim (lines 23, 31) this task partially falsifies for the light-extraction sub-surface
- `module/helper/renderer/src/webgl/loaders/gltf.rs` — the file this task edits (`light_list_get`, line 291)
- `module/helper/renderer/src/webgl/light.rs` — `Light`/`PointLight`/`DirectLight`/`SpotLight` type definitions this task's tests construct as expected values
- `module/helper/renderer/tests/gltf_loader_tests.rs` — the `asset_uri_resolve` precedent this task's promotion-and-export mechanism mirrors exactly

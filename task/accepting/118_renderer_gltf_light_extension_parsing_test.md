# Native test coverage for glTF KHR_lights_punctual light-extension parsing

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-19 00:46:27
- **expires_at:** 2026-08-19 02:46:27
- **round:** 1
- **state:** 🔎 (Accepting)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/renderer
- **verified_by:** system
- **verification_date:** 2026-08-16
- **blocked_by:** null
- **executing_at:** 2026-08-19 00:46:27
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **in_motion:** true
- **accepting_at:** 2026-08-19 00:46:27
- **accepting_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verified_at:** 2026-08-19 00:40:22

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

## Outcomes

### Acceptance Results

- **Verified by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ (independent acceptance-verification session — no prior memory of this task's implementation diff before this walk)
- **Date:** 2026-08-19
- **Verdict:** PASS

**B1 separation-of-concerns disclosure:** this verifying session's own resolved identity (`user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/`) shares the `user@host` prefix with the task's own `executing_by` value, the same mechanical collision already documented on tasks 202/246/247/248 this sweep. `tsk .acceptance_pass 118` is expected to mechanically refuse regardless of verdict; not forced or spoofed.

**Scope note:** no independent acceptance walk had previously been recorded for this task — only the pre-execution Readiness Gate (`## Verification Record` above) and one blocked `.acceptance_pass` attempt (Journal, 2026-08-17) existed. This is the first full Checklist/Measurements/Invariants/Anti-faking walk performed for task 118.

#### Checklist

- C1 — PASS — `pub fn light_list_get( gltf : &gltf::Gltf ) -> Option< FxHashMap< usize, Light > >` at `module/helper/renderer/src/webgl/loaders/gltf.rs:312`, with a `///` doc comment and `#[ must_use ]`.
- C2 — PASS — `light_list_get` present in the `mod_interface! { own use { ... } }` list (line ~1493), alongside `GLTF, load, required_extensions_check, asset_uri_resolve, light_list_get, light_get, ...`; reachable as `renderer::webgl::loaders::gltf::light_list_get` (confirmed by the test file's own `use` line).
- C3 — PASS — `module/helper/renderer/tests/gltf_light_parsing_test.rs` exists; the 4 Test-Matrix-mapped tests are present: `mixed_fixture_yields_three_lights_with_correct_fields` (T01), `point_light_missing_range_is_silently_skipped` (T02), `empty_lights_array_yields_some_empty_map` (T03), `missing_extension_key_yields_none` (T04).
- C4 — PASS — read `mixed_fixture_yields_three_lights_with_correct_fields` in full: it asserts index 0 is `Light::Point` with `color=[1,0,0]`, `strength=500.0`, `range=10.0`; index 1 is `Light::Direct` with `color=[0,1,0]`, `strength=2.5`; index 2 is `Light::Spot` with `color=[0,0,1]`, `strength=800.0`, `range=10.0`, `inner_cone_angle=0.1`, `outer_cone_angle=0.5` — full field-level assertions per variant, not variant-discriminant-only or map-length-only.
- C5 — PASS, with disclosure — `light_get` (the sibling function this task's own Out of Scope explicitly excludes) is in fact now `pub` and exported, and the test file contains 2 additional tests exercising it (`light_get_resolves_node_level_light_reference`, `light_get_derives_direction_from_rotation_not_translation`). Traced via `git log -S"pub fn light_get"` to commit `fbd3f206` (2026-08-16 09:42:46), a large batched commit ("Add comprehensive unit and integration tests... Create bug documentation files BUG-154 through BUG-175"). Both new tests' own doc comments cite `BUG-189` and `BUG-172` by number; `task/bug/completed/189_light_get_reads_wrong_gltf_extension_accessor.md` and `task/bug/completed/172_gltf_light_direction_from_position_not_rotation.md` both exist, state Completed, `verification_date: 2026-08-16` — same day, consistent with landing in the same batched commit as this task's own work. `task/completed/299_renderer_gltf_loader_pub_extraction_and_tests.md` (a separate, already-completed retroactive-documentation task) independently confirms task 118 as prior precedent for the pub-promotion pattern, not the origin of `light_get`'s own promotion. Conclusion: `light_get`'s pub/export/tests are BUG-189/BUG-172's own legitimate, separately-scoped deliverables that landed in the same file and commit as task 118's work, not scope creep by task 118 itself — task 118's own 4 Test-Matrix tests exercise `light_list_get` only.
- C6 — PASS — `git log -1 -- module/helper/renderer/src/webgl/light.rs` → last touched 2025-12-15, long before this task's 2026-08-16 execution window; `Light`/`PointLight`/`DirectLight`/`SpotLight` definitions untouched by this task.
- C7 — PASS — `git log -- module/helper/renderer/src/webgl/animation/loaders/gltf.rs` → most recent touches dated 2026-08-17/2026-08-18, after this task's own 2026-08-16 05:24–05:45 execution window; not touched by this task.
- C8 — PASS for this task's own 4 Test-Matrix tests (none reference `.position`/`.direction`), disclosed nuance: the file as a whole (post BUG-189/172 landing, see C5) now also contains 2 more tests that do assert `position`/`direction` — legitimately, as `light_get`'s own real computed output for those bugs' regression coverage, not as vacuous `Default`-pinning of `light_list_get`'s output (the concern the Out-of-Scope note actually raised). Task 118's own scope is unaffected.
- C9 — PASS — `light_list_get`'s own diff hunk (commit `fbd3f206`) changes only the signature (`fn` → `pub fn`) plus a new `///` doc comment and `#[ must_use ]`; the function body (the `for` loop over `gltf.lights()?.enumerate()`) is byte-identical to before. No `load`/`texture_upload`/`meshes_create`/`skeleton_load` body appears in that hunk.

#### Measurements

- M1 — PASS — `cargo test -p renderer --test gltf_light_parsing_test` (via mandatory `longrun` detached pattern, `-0002_longrun.log`) → exit 0, elapsed 178s: `test result: ok. 6 passed; 0 failed; 0 ignored` — 4 Test-Matrix tests + 2 BUG-189/172 regression tests, all passing.

#### Invariants

- I1 — PASS — `verb/test` via mandatory `longrun` detached pattern (`-0003_longrun.log`, exit 0, elapsed 1779s). Full log swept for `FAIL`/`error[`/`^error:`/`panicked`/`TIMEOUT` (excluding benign substring matches like a test named `..._not_panicked`) → zero hits. Native nextest: `2352 tests run: 2352 passed, 0 skipped`. Doc-tests: `2 tests run: 2 passed` ×2 blocks, all `0 failed`. wasm32 check: `56 example(s) checked, 0 failed`. wasm32 test: `4 crate(s) tested, 0 failed`.
- I2 — PASS — `RUSTFLAGS="--cfg web_sys_unstable_apis -D warnings" cargo check -p renderer --all-features` (via `longrun`, `-0004_longrun.log`) → exit 0; `grep -c "warning:"` on the full log → `0`.

#### Anti-faking checks

- AF1 — PASS — read the 3 `match` arms in `mixed_fixture_yields_three_lights_with_correct_fields` directly: `Light::Point` (`color=[1,0,0]`, `strength=500.0`, `range=10.0`), `Light::Direct` (`color=[0,1,0]`, `strength=2.5`), `Light::Spot` (`color=[0,0,1]`, `strength=800.0`, `range=10.0`, `inner_cone_angle=0.1`, `outer_cone_angle=0.5`) — 3 distinct variant constructions with pairwise-distinct field values, not one expected value repeated.
- AF2 — PASS — read `POINT_MISSING_RANGE_FIXTURE` directly: `{ "type": "point", "color": [1.0,1.0,1.0], "intensity": 1.0 }` — the `range` key is entirely absent (not present as `null` or `0.0`).

**Adversarial pass (dedicated, beyond the per-item checks above):** actively attempted to disprove each PASS above, focused on C5/C8 since those carry the only real ambiguity. (1) Checked whether `light_get`'s promotion could instead be task 118's own scope creep rather than BUG-189/172's — the batched commit's message and the two tests' own explicit `BUG-189`/`BUG-172` doc-comment citations, cross-referenced against both bugs' own `task/bug/completed/` files with matching `verification_date: 2026-08-16`, make independent-origin the better-supported reading; no evidence (no task-118-authored doc comment, no Test-Matrix row) ties `light_get`'s promotion to task 118 itself. (2) Checked whether task 118's own Test Matrix might be silently incomplete now that the file has 6 tests instead of 4 — re-read the Test Matrix (T01-T04) against the 4 correctly-mapped test names; all 4 rows have exactly one corresponding passing test, no row is double-counted against a `light_get` test. (3) Checked I1's wasm32 "no tests to run" pattern for `gltf_light_parsing_test.rs` specifically — confirmed absent from the log's wasm32 section entirely (the crate's tests run natively only, no `#[ wasm_bindgen_test ]` markers in the file, consistent with M1 already exercising all 6 cases natively). No blocking finding surfaced.

**BUG-197 mechanical guard (upfront disclosure):** per the B1 disclosure above, `tsk .acceptance_pass 118` is expected to refuse this transition (exit 1, "self-verification forbidden (actor matches executing_by)") since this verifying session's `scope get::id` shares the `user@host` prefix with the task's own `executing_by` field. No override was requested or authorized; the CLI's actual exit code and message will be reported verbatim in the Journal below; no Execution State field will be hand-edited to force closure.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-16 05:24:13 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | CLAIM_EXEC | execution claimed |
| 2026-08-16 05:45:03 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | EXEC_COMPLETE | execution complete |
| 2026-08-16 05:45:09 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-17 00:49:51 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | ATTEMPT_ACCEPTANCE_PASS | `tsk .acceptance_pass 118` → exit 1, "self-verification forbidden (actor matches executing_by)" — same-actor sandbox guard, consistent with task 202/206 precedent; not forced/spoofed, left at 🔎 Accepting per standing project convention |
| 2026-08-19 00:40:22 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 00:46:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-19 00:46:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | EXEC_COMPLETE | execution complete |
| 2026-08-19 00:46:27 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-19 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | ATTEMPT_ACCEPTANCE_PASS | `tsk .acceptance_pass 118` → exit 1, "self-verification forbidden (actor matches executing_by)" — same-actor sandbox guard, consistent with prior 2026-08-17 attempt and this sweep's 202/246/192/247/248 precedent; not forced/spoofed, left at 🔎 Accepting with PASS verdict documented in `### Acceptance Results` above per standing project convention |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-16]** `FILED` — Task filed via `/doc_tsk` Phase 3 (docs/layer gap audit): add native test coverage for glTF `KHR_lights_punctual` light-extension parsing to `renderer`.
- **[2026-08-19]** `NOTE` — No `EXECUTED` entry was ever recorded despite the Journal showing `EXEC_COMPLETE`; the underlying work is real and verified this session. Confirmed live: `light_list_get` is `pub fn` (`gltf.rs:312`) and present in the `mod_interface!` `own use` list; `tests/gltf_light_parsing_test.rs` exists; ran `cargo test -p renderer --test gltf_light_parsing_test` directly → 6/6 passed (M1/I1 real evidence). T01-T04's own 4 tests (`mixed_fixture_yields_three_lights_with_correct_fields`, `point_light_missing_range_is_silently_skipped`, `empty_lights_array_yields_some_empty_map`, `missing_extension_key_yields_none`) map cleanly and pass. **C5 will read FAIL on a literal walk** — `light_get` (this task's own named Out-of-Scope item) is now also `pub` and exported, and the test file's other 2 tests (`light_get_resolves_node_level_light_reference`, `light_get_derives_direction_from_rotation_not_translation`) exercise it. Traced via `git log -S"pub fn light_get"`: both changes landed in commit `fbd3f206` ("feat: add test coverage expansion and bug documentation across modules"), a large unrelated concurrent-actor sweep — not this task's own execution overreaching. Confirmed the two `light_get` tests are reproducers for BUG-172 and BUG-189, both already filed, fixed, and closed (`task/bug/completed/172_gltf_light_direction_from_position_not_rotation.md`, `task/bug/completed/189_light_get_reads_wrong_gltf_extension_accessor.md`) — legitimate, already-tracked work that happened to land in the same file, not a stray regression. Left `light_get`'s pub/export status as-is (no revert) — it has no other external callers today (confirmed via repo-wide grep), reverting a days-old, already-relied-upon-by-tests visibility change carries its own risk, and the underlying bug fixes it enabled are real and closed. `tsk .acceptance_pass 118` already documented blocked in `## Journal` (2026-08-17, same-actor guard) — not re-attempted, no reason to expect a different result.

## Related Documentation

- `docs/layer/005_l4_scene_model.md` — the layer doc whose blanket "requires a live GPU context just to parse" claim (lines 23, 31) this task partially falsifies for the light-extraction sub-surface
- `module/helper/renderer/src/webgl/loaders/gltf.rs` — the file this task edits (`light_list_get`, line 291)
- `module/helper/renderer/src/webgl/light.rs` — `Light`/`PointLight`/`DirectLight`/`SpotLight` type definitions this task's tests construct as expected values
- `module/helper/renderer/tests/gltf_loader_tests.rs` — the `asset_uri_resolve` precedent this task's promotion-and-export mechanism mirrors exactly

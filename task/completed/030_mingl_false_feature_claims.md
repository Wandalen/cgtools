# Fix mingl's false feature claims in docs

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
- **unit:** lib/yrd_gamedev/cgtools/module/min/mingl
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

`mingl`'s documentation claims a feature or capability the audit found not actually implemented in the
crate's real source (P5 — remaining doc drift, Fix-in-place). **Exact claim and file were not preserved
precisely through this session's context compaction — re-derive by diffing the crate's readme/doc claims
against `src/` at pickup** (note: `module/min/mingl/src/web/exec_loop.rs` is the file task 012 confirms
minwebgl should be reusing — check that file's own doc comments for accuracy while in this crate, since
it's directly relevant). Kept as a separate task from task 029 (browser_log's own false claims) per Crate
Scope Unity even though both were found in the same audit pass.

## Verification

### Checklist

- [x] C1 — Is `readme.md` (the crate's `include_str!`'d doc root, `src/lib.rs:7`) free of the fictional `CameraController` trait name? `grep -c "CameraController" module/min/mingl/readme.md` → `0`.
- [x] C2 — Is `readme.md` free of the fictional `mingl::convert` module path? `grep -c "mingl::convert" module/min/mingl/readme.md` → `0`.
- [x] C3 — Is `readme.md` free of the fictional `projection_matrix()` / "Perspective & Orthographic" projection-mode claims? `grep -cE "projection_matrix|Perspective & Orthographic" module/min/mingl/readme.md` → `0`.
- [x] C4 — Does `readme.md` document the real `CameraOrbitControls` API (`rotate()/pan()/zoom()/update()/view()`) as it exists in source today? Confirmed: `readme.md:102` lists exactly these 5 methods, and all 5 are implemented on `CameraOrbitControls`. Note: the source file cited in this task's History (`camera_orbit_controls.rs:196-204`, a flat top-level path) is now at `src/controls/camera_orbit_controls.rs` (struct at line 197) — confirmed already at this nested path as of this task's own completion baseline (`git ls-tree -r --name-only 25ceae76 -- module/min/mingl/src | grep camera_orbit` → `module/min/mingl/src/controls/camera_orbit_controls.rs`), so the History text's flat-path/line citation was stale from the moment this task's doc fix landed, not from any later drift. The API substance is unchanged.
- [x] C5 — Does `readme.md` correctly state degrees (not radians) for the rotation-constraint setters, per this task's own in-loop B5 fix? `grep -n "angles in degrees" module/min/mingl/readme.md` → line 125 present; `src/controls/camera_orbit_controls.rs`'s `base_longitude_set`/`longitude_range_set`/`base_latitude_set`/`latitude_range_set` all clamp in degrees (e.g. `.clamp( 0.0, 360.0 )`), matching the doc.
- [x] C6 — Is the real `asbytes` re-export (cited in History as `mem.rs:133`) still live? Confirmed: `src/mem.rs` (now 9 lines total — reduced from 143 by task 061 after this task ran) contains `reuse ::asbytes;` at line 8.
- [x] C7 — Does `readme.md` document the real `CharacterControls` (this task's replacement for the fictional `CameraController`)? `grep -c "CharacterControls" module/min/mingl/readme.md` → `3`; `src/controls/character_controls.rs` implements `position()/yaw()/pitch()/forward_xz()/right_xz()` matching the documented quick-start snippet.
- [x] C8 — Does `readme.md` document real OBJ model reporting (`BoundingBox`/`BoundingSphere`, `model_obj` feature) rather than the fictional API? Confirmed: both structs with a `compute()` constructor exist in `src/model/obj.rs`, matching `readme.md`'s "OBJ Model Reporting" section.
- [x] C9 — Is `readme.md`'s "Primitive Coverage" claim (the one task 061 later corrected from f32-only) still an accurate, non-stale description of current source? `readme.md:11` states nested `[[T; M]; N]` support "for all supported scalars"; confirmed all 7 `src/data_type/*.rs` files (f32/i8/i16/i32/u8/u16/u32) each carry exactly one nested-array `IntoVectorDataType` impl today.

### Measurements

- [x] M1 — Combined fictional-term count in `readme.md` (`CameraController`, `mingl::convert`, `projection_matrix`): current `0` (was: `5`, `git show 25ceae76:module/min/mingl/readme.md | grep -cE "CameraController|mingl::convert|projection_matrix"`).
- [x] M2 — `readme.md` total line count: current `264` (was: `239`, `git show 25ceae76:module/min/mingl/readme.md | wc -l`).

### Invariants

- [x] I1 — Test suite (crate-scoped): `cargo nextest run -p mingl --all-features` → exit 0, 51/51 passed (13 inline `web::file` unit tests + 38 `tests/tests/*.rs` integration tests).
- [ ] I2 — Compiler/lints clean (crate-scoped): `cargo clippy -p mingl --all-targets --all-features -- -D warnings` → exit 101, NOT clean. Root cause fully isolated to a different, workspace-local crate: `module/helper/browser_log/src/panic.rs:82`'s `#[ allow( clippy::exhaustive_structs ) ]` lacks a `reason = ".."`, tripping the workspace's `allow_attributes_without_reason = "warn"` lint (escalated to a hard error by `-D warnings`). `browser_log` is pulled in only transitively, via mingl's optional `web_log` feature; the build aborts there before mingl's own source is ever clippy-checked. `git log -1 --format="%h %ad %s" --date=iso -- module/helper/browser_log/src/panic.rs` → commit `5f33be66`, dated 2026-08-11 (today) — lands after this task's 2026-08-10 completion and touches none of the files this task changed, so this is pre-existing drift unrelated to task 030, not a regression it introduced. (Independently corroborated: a concurrent sibling verification of the unrelated `primitive_generation` crate hit the identical `browser_log:82` failure in the same session.)

### Anti-faking checks

- [x] AF1 — Guards against the fictional-API inventory silently reappearing under a different heading: residue grep for the full fictional-name set from this task's History (`CameraController|mingl::convert|InputState|MouseButton|rotate_around_target|projection_matrix|Perspective & Orthographic|keyboard`) against `readme.md` → `0` hits for every term today; a future readme edit reintroducing any of these without a matching real source symbol is the same fiction this task removed.
- [x] AF2 — Guards against the degrees/radians correction (C5, this task's own B5 in-loop catch) silently reverting: `grep -n "angles in degrees"` on `readme.md` must keep returning a hit paired with the setters' actual `.clamp(...)` behavior in `src/controls/camera_orbit_controls.rs`; a future radians claim with no corresponding setter-range change reintroduces the exact defect this task's gate check caught before completion.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P5 (doc drift)
  tier, Fix-in-place bucket. Flagged: citation detail needs re-derivation at pickup.
- **[2026-08-10]** `IMPLEMENTED` — Re-derivation located the drift in `readme.md` (the file the crate
  `include_str!`s as its doc root): the Camera System and Data Conversion narratives were majority
  fiction. Fictional API documented as real: `Camera`/`OrbitControls` types with builder methods
  `.position()/.target()/.up()/.fov()/.near()/.far()`, `controls.update( &mut camera, dt )`,
  `view_matrix()`, `projection_matrix()`, "Perspective & Orthographic" projection modes, keyboard input,
  `mingl::convert::{ IntoVector, IntoBytes }`, `into_vector()`, `Vec<T>` descriptor support, a
  `CameraController` trait with `InputState`/`MouseButton`/`rotate_around_target()`. Real API verified in
  source: `CameraOrbitControls` (public fields `eye`/`up`/`center`/`window_size`/`fov`,
  `camera_orbit_controls.rs:196-204`) with `rotate()/pan()/zoom()/update()/view()` (view = right-handed
  look-at; no projection code exists anywhere), constraint state structs with `_set` accessors clamping in
  degrees (lines 57-115), `bind_controls_to_input( &HtmlCanvasElement, &Rc<RefCell<..>> )` (line 557,
  `web` feature, mouse + touch, no keyboard), `IntoVectorDataType`/`VectorDataType` descriptors
  (`data_type.rs`; per-primitive scalar + `[T; N]` impls, nested `[[f32; M]; N]` for f32 only, no `Vec`
  impls), and — decisively — `mem.rs:133` `reuse ::asbytes;` is LIVE, re-exporting real
  `AsBytes`/`IntoBytes` (asbytes 0.2.0, `Pod`-based, `Vec<T>`/`[T; N]` impls) at crate root, confirmed by
  examples (`use gl::{ GL, ..., AsBytes }`). `exec_loop.rs` doc comments checked per Goal note — accurate.
  Rewrote features, quick starts, API tables, configuration, advanced sections, and the WebGL example to
  the verified API in house codestyle; replaced the `CameraController` fiction with the real
  `CharacterControls` and added real OBJ model reporting (`BoundingBox`/`BoundingSphere`, `model_obj`
  feature); byte-conversion examples kept (substance was real) with the fictional `mingl::convert` path
  corrected. Verified: `cargo test -p mingl` exit 0 (12 unit tests passed, 8 readme doc blocks ignored as
  `rust,ignore`), residue grep for all fictional names clean.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Tier 2 dual-role gate check passed 15/15. In-loop
  adversarial catches: (1) initial inventory classed "Byte Slice Utilities" as wholly false because
  `mem.rs`'s own `AsBytes` trait is commented out — the adversarial dependency sweep found the live
  `reuse ::asbytes;` re-export, preventing an overcorrection that would have deleted TRUE byte-conversion
  claims; (2) my replacement Camera Configuration text initially said "angles in radians" — source doc
  comments clamp in degrees ([0,360]/[0,180]/[-90,90]); fixed before completion.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | Goal's re-derivation mandate satisfied: drift located in readme.md, full claim-by-claim inventory built | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value / YAGNI | 🟢 | 🟢 | — | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | Single file edited (readme.md); exec_loop.rs checked read-only per Goal note | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | mingl only | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | Snippets rewritten in house codestyle (2-space, spaced parens) | — |
| B2 | Test-First | 🟢 | 🟢 | Doc-drift task: source is the oracle; every replacement claim source-anchored with line evidence | — |
| B3 | Evidence of Failure | 🟡 | 🟢 | Adversarial sweep found `reuse ::asbytes;` live at mem.rs:133 — byte claims were substance-TRUE, path-fictional; initial wholly-false classification was itself wrong | Kept byte examples, fixed import path to real root re-export |
| B4 | Proper Fix Only | 🟢 | 🟢 | No hedging left; fiction replaced by verified API, not deleted wholesale | — |
| B5 | Fix Verification | 🟡 | 🟢 | Own replacement text claimed radians; setters clamp in degrees per doc comments (lines 57-115) | Corrected to degrees with explicit clamp ranges |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Completion record carries full fiction inventory + real-API map with line citations | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | Residue grep clean (OrbitControls, view_matrix, projection_matrix, IntoVector, convert, CameraController, keyboard); cargo test -p mingl exit 0 | — |
| **Total** | | 🔴 | 🟢 | 2 findings resolved | 2/2 |

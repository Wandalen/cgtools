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

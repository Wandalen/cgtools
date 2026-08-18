# Falling Frontier → cgtools port: working plan

Source of truth for progress on this port. Update this file as milestones land —
it's meant to survive context resets, so write for a reader who has none of the
current conversation.

## Resume here

**All milestones (M0-M8) are done, verified, and committed.** This port is
feature-complete against its own plan - see each checklist entry below for
what/how (M7's verification hit a real testing-environment limit worth
reading before touching fleet motion again; M8's is the last entry).
- `851dd9df` on `space-game-demo` — M0-M3 ("feat: add Falling Frontier
  tactical grid, dev panel, and view-zone ribbon")
- `6c71a5c8` on `space-game-demo` — M4 ("feat: add Falling Frontier ships,
  station, and starfield (M4)")
- `0a8434ea` on `space-game-demo` — M5 ("feat: add Falling Frontier real
  object picking/selection (M5)")
- `23f85330` on `space-game-demo` — M6 ("feat: add Falling Frontier
  transform gizmo (M6)")
- `5971b6a0` on `space-game-demo` — M7 ("feat: add Falling Frontier fleet
  motion and trajectories (M7)")
- `50ef62de` on `space-game-demo` — M8 ("feat: add Falling Frontier
  tactical HUD (M8)")

All with no `Co-Authored-By` trailer, per the standing repo rule (see memory
`feedback_commit_trailers`). `examples/minwebgl/falling_frontier/Untitled.png`
is an untracked debug screenshot the user pasted in during the M4 starfield
investigation (see Notes section below) — left untracked on purpose, safe to
delete once no longer needed, not part of the deliverable.

**Post-milestone work (uncommitted as of this note, pending user review):**
- **Real bug fixed in `line_tools`** (not this crate): `d3::Line::mesh_create`
  (and `d2::Line`'s join/cap mesh setup) uploaded the index buffer *before*
  creating/binding its own VAO. `ELEMENT_ARRAY_BUFFER` binding is part of the
  *currently bound VAO's* state in WebGL2 (unlike `ARRAY_BUFFER`, which is
  global), so the upload silently overwrote whatever VAO was bound
  previously — in this app, the station beacon's icosphere VAO, which then
  failed `drawElements` every frame with `GL_INVALID_OPERATION: Insufficient
  buffer size` once trajectories/sensor rings were ever constructed. Fixed by
  moving the index upload after VAO creation in all three call sites. See
  `module/helper/line_tools/src/d2/line.rs` and `d3/line.rs`.
- **Promoted three modules out of this example into shared `module/`
  crates**, per the standing "extend cgtools crates" instruction and the
  example-first-then-promote pattern:
  - `primitives.rs` (box/cylinder/torus/icosphere generators) →
    `primitive_generation::{box_mesh, cylinder_mesh, torus_mesh, icosphere}`
    (new `src/solid.rs` layer — raw `(positions, indices)` pairs, deliberately
    *not* funneled through `PrimitiveData`/`primitives_data_to_gltf`, since no
    consumer wants the GLTF pipeline for these).
  - `spline.rs` (Catmull-Rom path evaluation) →
    `primitive_generation::{point_at_progress, tangent_at_progress}` (new
    `src/spline.rs` layer, unit tests carried over verbatim and still pass).
  - `picking.rs` (GPU id-buffer picking) → new crate `module/helper/gpu_picking`
    (`IdProgram`, `PickBuffer`), generalized via a `Pickable` trait (`vao`,
    `index_count`, `model`, `pick_id`) instead of being coupled to this
    crate's own `HullPart` — `hull.rs` now does `impl gpu_picking::Pickable
    for HullPart`. `examples/minwebgl/object_picking` still has its own
    inline copy of this same pattern and was *not* migrated onto the new
    crate in this pass (out of scope, no functional need); worth doing if
    that example gets touched again.
  - All three verified: `cargo check`/`clippy` clean for `gpu_picking`,
    `primitive_generation`, and `falling_frontier` (wasm target); live in
    browser — ship/station/asteroid picking, gizmo translate+rotate (through
    the promoted `box_mesh`/`torus_mesh`), and the fleet patrol spline all
    re-verified working with zero console errors after the move.

**If picking this back up**: there's no "next task" - re-read the gap audit
(`research/falling_frontier_cgtools_audit.md`) against what actually landed
to decide what (if anything) is still worth doing. Candidates already
flagged in this file as deliberately cut, in case priorities change:
per-axis gizmo handles instead of one free-drag handle each for translate/
rotate (M6), waypoint ring markers + height-guide-lines on the trajectory
ribbon (M7), and the JS reference's inert decoration - mission objectives,
fake resources readout, tactical-module buttons, hull/shield/thrust status
bars, subsystem-matrix footer, bottom toolbar icons (M8, all of it dead
weight with zero wired behavior even in the JS reference itself).

Reference material:
- Gap audit: `research/falling_frontier_cgtools_audit.md` (76-feature comparison,
  render-pipeline flexibility finding)
- JS original: `examples/threejs/falling_frontier/src/` — port faithfully from
  here unless a note below says otherwise
- Standing instruction from the user: **extending/adding cgtools crates is in
  scope and is the point of this port**, not a last resort. Build inside this
  example crate first while a piece is still being proven out, promote to a
  shared `module/helper/*` or `module/shader/*` crate once its shape is proven
  (same pattern `line_tools`/`primitive_generation` already followed) — see
  memory `project_falling_frontier_port` for the fuller version of this note.
- No `Co-Authored-By` trailer on any commit for this work (standing repo rule).

## Milestone order (risk-first: grid shader before the rest of the visible scene)

- [x] **M0** — scaffold: orbit camera (`renderer::webgl::Camera` +
      `mingl::CameraOrbitControls`), full-viewport canvas, placeholder
      ground/grid (now superseded by M1's real shader).
- [x] **M1** — base tactical grid shader: `fwidth`-based analytic-AA lines +
      camera-distance fade, ported from `tacticalGrid.js`. Standalone draw
      call (NOT a `Material` impl — WBOIT forces a two-output shader contract
      on any `AlphaMode::Blend` `Material`; see audit §"Rendering Pipeline
      Architecture"). Files: `src/shaders/grid.{vert,frag}`, `TacticalGrid` in
      `main.rs`.
- [x] **M2** — dev tuning panel. DOM-based via web-sys (egui/winit don't
      exist anywhere in this workspace — see audit's Dev Tooling section),
      ported from `gridTuningPanel.js`. Scope: only the uniforms M1 actually
      wired (line color/width, cell size, dim alpha, camera-fade
      start/end/mode/gamma) — NOT the ribbon/glow controls from the JS panel,
      since M3 hasn't landed yet. **Extend this panel when M3 adds those
      uniforms rather than building a second one.** Files:
      `src/debug/{mod.rs,grid_tuning.rs,grid_tuning_panel.rs}`. State lives in
      `Rc<RefCell<GridTuning>>`, shared between the panel's DOM closures and
      the per-frame render loop (`TacticalGrid::draw` now reads live values
      instead of baking constants in at construction). Verified live: all
      sliders/select/color picker update the grid in real time, Copy Settings
      writes a paste-back-able summary to the clipboard successfully.
- [x] **M3** — view-zone ribbon + boundary polyline + point-in-polygon +
      asteroid glow. Files: `src/boundary.rs` (CPU polyline builder, ported
      1:1 from `buildBoundaryPolyline`/`sampleBoundaryRadius`/
      `normalizeAngle`), `src/asteroids.rs` (procedural rock geometry +
      rendering + `blockers()`/`glow_candidates()` queries), `grid.frag`
      (extended with the ribbon/inside-fade/glow uniforms and logic, ported
      1:1 from the JS fragment shader), ground-click handling + ray/plane
      unprojection in `main.rs` (`ray_ground_hit`/`unproject`/
      `setup_ground_click`). Tuning panel extended with Focus/Ribbon/Glow
      sections. Verified live: clicking the ground sets a focus point, the
      ribbon renders as a faceted circle that wraps tight around any
      asteroid within view range (confirmed visually — sharp inward notches
      at each blocker, exactly the JS "taut string" look), a click far from
      any asteroid renders a clean unwrapped circle, dragging the camera
      (orbit) does NOT spuriously set focus (6px click-vs-drag threshold),
      Clear Focus removes the ribbon, no console errors/panics during normal
      frames. Deliberate simplification vs. JS: asteroid geometry is a
      jittered icosahedron (20 faces) rather than a jittered dodecahedron
      (12 faces, three.js `detail=1`) — purely cosmetic difference, doesn't
      touch the boundary/glow math (see asteroids.rs's module doc for the
      full reasoning); revisit only if it reads as too rough once M4 adds
      real ships for scale comparison.
- [x] **M4** — full static scene content: ship hulls, station, starfield.
      Files: `src/primitives.rs` (box/cylinder-or-cone/torus/icosphere
      generators, built here rather than in `primitive_generation` per the
      example-first-then-promote pattern — none existed anywhere in
      cgtools before this), `src/hull.rs` (one shared flat-shaded-via-
      derivatives program + `HullPart`/`HullProgram`, generalized from M3's
      asteroid-only shader so ships/station/asteroids all draw through it),
      `src/ships.rs` (4-ship fleet, box+cone+cylinder compositing, ported
      from `ships.js`/`fleet.js`), `src/station.rs` (core/ring/spokes/
      docking-modules/beacon, ported from `spaceStation.js`),
      `src/starfield.rs` (2000-point dust field, own small unlit
      point-sprite program, ported from `starfield.js`). Scope is static
      placement only — `fleet.js`'s patrol paths, trajectory ribbons, and
      `spinStations`/asteroid idle-spin are explicitly M7/M4-polish, not
      built here. Verified live: ships/station/starfield/asteroids all
      render together, M3's ribbon still correctly wraps only around
      asteroids (ships/station aren't registered as blockers, matching the
      JS reference), no console errors. See the starfield-clustering
      debugging notes below - resolved before committing, not a
      known-remaining issue.
- [x] **M5** — real object picking/selection, replacing M3's ground-click
      stand-in. Files: `src/picking.rs` (`IdProgram` + `PickBuffer`, ported
      from `examples/minwebgl/object_picking`'s GPU-ID-buffer pattern -
      adapted to upload `u_view_proj * u_model` per part instead of that
      example's origin-fixed-camera projection-only math, and to resize
      alongside the canvas instead of a fixed 1280x720), `src/shaders/
      id.{vert,frag}`. `hull.rs`'s `HullPart` gained a `pick_id : i32` field
      (asteroids: one id per rock; ships: every part of one ship shares that
      ship's id, since a click anywhere on a composited ship should select
      the whole ship; station: every part shares the one station id) and
      `HullProgram::draw_part` gained a `highlighted : bool` param that mixes
      the part's color toward white and forces full ambient - a color-tint
      selection indicator, not a geometric outline (no gizmo exists yet to
      make selection obvious some other way; M6 will add one). `main.rs`
      assigns contiguous id ranges (`ASTEROID_ID_BASE`/`SHIP_ID_BASE`/
      `STATION_ID`, sized from `asteroids::ASTEROID_COUNT`/`ships::
      SHIP_COUNT` so they can't drift out of sync with the spec arrays) and
      classifies a picked id back into `PickedKind::{Asteroid,Ship,Station}`.
      Click handling was renamed `setup_ground_click` → `setup_selection_click`
      and now re-renders the id pass + reads back one pixel on every
      qualifying click (not cached - see the note on why, below) instead of
      ray/plane-intersecting the ground; `main.rs`'s old `unproject`/
      `ray_ground_hit` functions were deleted as a result (M6 will need
      similar math for gizmo dragging - see "Next task" above and the
      preserved math under Notes). The grid's view-zone ribbon is no longer
      driven by a stand-in ground click at all - it's derived fresh every
      frame from whatever is currently selected, matching the *actual* JS
      reference's own `main.js` `animate()` (which points the ribbon at
      `gizmo.object` only when it defines a `viewRadius`, i.e. only ships;
      the station/asteroids have none). `debug/grid_tuning_panel.rs` no
      longer depends on `FocusState` at all - `refresh_selection_status`
      takes a plain caller-built `&str` and the "Clear Focus" button became
      "Deselect", wired through an `on_deselect` callback the panel invokes
      rather than reaching into app state itself. Verified live: clicking
      each of a ship/the station/an asteroid selects it (tint highlight
      visible, panel shows "selected: ship 1" / "selected: station" /
      "selected: asteroid 0"), only the ship selection shows the view-zone
      ribbon (station/asteroid selection highlights but no ribbon, matching
      the JS), clicking empty space or the grid deselects, the Deselect
      button deselects, no console errors across a hard refresh + repeated
      clicks.
- [x] **M6** — transform gizmo (translate XZ / rotate Y, G/R/Escape). No
      gizmo existed anywhere in cgtools before this. Files: `src/gizmo.rs`
      (`Gizmo`/`GizmoMode` - two simplified handle meshes, a flat XZ "move"
      cross and a flat "rotate" ring, drawn with depth test off so the
      handle stays visible/clickable through the object it belongs to; each
      mode's handle is the object's *only* handle, not one arrow per axis
      like the JS reference's `TransformControls` - translate is XZ-only in
      this scene anyway (`MODE_CONSTRAINTS`'s `showY: false`), so a single
      free-drag-in-plane handle covers the same freedom without per-axis
      hit-testing), `src/shaders/gizmo.{vert,frag}` (unlit, flat color +
      alpha). `hull.rs`'s `HullPart` gained a `local_transform` field (the
      part's fixed offset within its object) plus `set_model` (recomputes
      `model` from a new object-level transform) - needed because dragging
      requires an object's `model` matrices to be *recomputed*, not just
      overwritten, since a ship's `model = ship_transform * local_transform`
      was previously baked into one matrix with no way to recover
      `ship_transform` alone. `asteroids.rs`/`ships.rs`/`station.rs` each
      gained mutable per-object state (`AsteroidObject`/`ShipObject`/
      `Station`'s own `position`/`rotation_y` fields, mutated by new
      `drag_to`/`rotate_to`/`object_transform` methods) - `blockers()`/
      `glow_candidates()`/`position()` all read the *live* (possibly
      dragged) position now, so M3's ribbon/glow correctly follow a dragged
      asteroid or the selected ship. `main.rs` re-derives `unproject`/
      `ray_ground_hit` (deleted in M5, preserved in this file's Notes
      section for exactly this) for drag math, adds a `DragState` (captured
      grab offset/angle so a drag doesn't snap the object to the cursor) and
      folds everything selection/picking/gizmo-related into one
      `InteractionCtx` behind a single `Rc` (mirroring
      `examples/minwebgl/object_picking`'s own `RenderCtx` pattern) so the
      pointerdown/pointermove/pointerup/keydown listeners (`
      setup_selection_and_gizmo`, replacing M5's `setup_selection_click`)
      each capture one clone instead of a dozen separate `Rc`s. Camera-orbit
      rotation is suppressed during a drag via `Camera::controls_get()`'s
      shared `Rc<RefCell<CameraOrbitControls>>` (`rotation.enabled = false`)
      - the same mechanism the JS reference's `transform.js` uses
        (`world.controls.enabled = !event.value`). G/R/Escape are window-scoped
      `keydown`, matching the JS reference's own `window.addEventListener`.
      Verified live: selecting a ship/station/asteroid shows the yellow
      translate cross at its position; dragging it moves the object exactly
      along the cursor (camera does NOT orbit mid-drag); pressing R swaps to
      the magenta rotate ring, dragging it turns the object smoothly around
      Y; Escape deselects (removing the handle); repeated select→drag→
      deselect cycles across all three object kinds with no console errors.
      One real bug caught and fixed during this verification - see the
      "gizmo handle lost the depth test" note below, it's the kind of thing
      that will bite again if `PickBuffer::render` grows a third kind of
      always-on-top overlay later.
- [x] **M7** — fleet motion + trajectories. Files: `src/spline.rs` (uniform
      Catmull-Rom point/tangent evaluation - a new standalone module rather
      than an extension of `module/helper/animation`'s Hermite/Bezier
      easing, since that crate's abstractions are for 1D scalar `t -> value`
      easing curves, a different concern from 2D spline-point evaluation;
      ships six unit tests, see below), `src/trajectories.rs` (`Trajectories`
      - one solid ribbon `line_tools::d3::Line` per ship sampled from the
      same spline, one dashed sensor-ring `Line` per ship that defines a
      `sensor_radius` - `line_tools`'s first user in this crate, exactly the
      "handful of rings/paths" M1's own note reserved it for). `ships.rs`
      gained per-ship `path`/`speed`/`trajectory_color`/`sensor_radius`
      spec fields (ported from `fleet.js`), a `progress` field on
      `ShipObject`, and `advance(index, delta_progress)` - wraps past `1.0`
      back to `0.0` (matching the JS reference's own `if (progress > 1)
      progress = 0`), moves the ship to `spline::point_at_progress`, and
      orients it along `spline::tangent_at_progress` using the same
      `x.atan2(z)` convention the M6 gizmo's rotate-drag already uses and
      verified visually. `main.rs`'s render loop calls `advance` for every
      ship *except* whichever one is currently selected, each frame -
      matches `main.js`'s `updateFleetMotion` skipping its `excludeMesh`
      (the gizmo-attached ship) so a drag isn't fought by the path
      animation; checked GPU id-buffer picking's own staleness concern
      (flagged in M6's own notes) and confirmed it's already fine as
      written - `pick_at_client` re-renders the id pass fresh from current
      state on every call, never a cached/stale texture, so a moving ship
      is always picked at its position *as of that click*, no change
      needed. Playback is gated behind three new `GridTuning` fields
      (`animate_ships`/`show_trajectories`/`show_sensor_rings`, all
      `false` by default, extending the dev panel again rather than
      building a second one - matches `playbackState.isAnimating: false`
      and `groups.trajectory.visible = false; groups.sensorRing.visible =
      false;` in the JS reference, which are *also* off by default at this
      point in its own development). Deliberate simplifications vs. the JS
      reference, none affecting correctness: uniform (not centripetal)
      Catmull-Rom parametrization and segment-uniform (not arc-length-
      corrected) `t`-to-position mapping (see `spline.rs`'s own module doc
      for why neither matters for these short, evenly-spaced paths);
      waypoint ring markers and per-waypoint dashed height-guide-lines
      dropped as decorative flourishes on top of a ribbon that's already
      there (`trajectories.rs`'s module doc); no fast-forward/pause
      transport (`speed_multiplier` fixed at `1.0` - that's M8 UI chrome,
      not core motion). **A real bug was caught by unit tests, not by
      visual testing** - see the dedicated note below; visual testing
      itself hit a genuine environment limitation (`document.hidden` fully
      suspending `requestAnimationFrame`, so continuous motion couldn't be
      *watched* happen), also detailed below. What *was* confirmed live:
      trajectory ribbons and dashed sensor rings render correctly and
      match their expected shapes once the panel checkboxes are ticked, the
      one-time reorientation snap when enabling animation lands ships
      facing along their path's initial tangent, selection/gizmo/drag
      (M5/M6) all continue to work unmodified with animation enabled, and
      no console errors across extensive interaction.
- [x] **M8** — tactical HUD: status bar, unit-info card, view-layer toolbar,
      CRT scanline overlay, camera reset. Files: `src/hud.rs` (all of it -
      built via raw DOM calls, same as `debug/grid_tuning_panel.rs`, for the
      same reason). Ported from `index.html` + `src/style.css` +
      `interaction/uiControls.js` + `ui/unitPanel.js`, with a real CDN
      dependency dropped and a large chunk of the JS reference's own markup
      deliberately cut - both explained in full in `hud.rs`'s own module
      doc, summarized here:
      - The JS reference pulls Tailwind and Google Fonts from CDNs
        (`<script src="https://cdn.tailwindcss.com">` et al.); this crate
        stays self-contained (no CDN dependency, consistent with everything
        else in it), so `hud.rs` ships its own plain-CSS `<style>` block
        reproducing just the classes the ported markup actually uses, with a
        system font stack standing in for `Rajdhani`/`Share Tech Mono`.
      - Cut entirely: mission-objectives checklist, fake resources/credits
        readout *content* (the readout's static text is kept, see below),
        unit card tactical-module buttons (uplink/jammer/targeting/
        torpedo), hull/shield/thrust status bars, subsystem-matrix footer,
        bottom toolbar's category icons - **none of these are wired to any
        real state even in the JS reference itself** (`uiControls.js`/
        `unitPanel.js` never attach a listener to any of them), so porting
        them would just be inert decoration with zero behavior.
      - Kept as static flavor text despite being non-functional: the date/
        stardate and location/resources readout in the top bar - equally
        static in the JS reference (never updated by any code), ported 1:1
        anyway since a "status bar" was explicitly asked for and these cost
        nothing to keep.
      `ships.rs` gained `name`/`commander` spec fields + `name()`/
      `commander()`/`class_label()` accessors (ported from `fleet.js`);
      `station.rs` gained the equivalent `STATION_NAME`/`STATION_COMMANDER`
      consts + accessors (from `spaceStation.js`'s `STATION_SPEC`).
      Asteroids have no JS-side name/commander at all (the JS reference's
      own `selectUnit` is only ever called for objects with a
      `userData.name`, which asteroids never get - selecting one there
      doesn't open the unit card); this port shows a generic "ASTEROID n"
      contact instead of hiding the card for asteroids specifically, since
      M5/M6 already treat all three pickable kinds uniformly everywhere
      else and hiding the card only here would be a pointless
      inconsistency (`main.rs`'s `unit_info_for`). `GridTuning` gained
      `show_grid` (default `true`) and `speed_multiplier` (default `1.0`,
      set to `2.5` by the HUD's Fast button) - the render loop now gates
      `grid.draw` behind `show_grid` and scales each ship's per-frame
      progress step by `speed_multiplier`. The HUD's own toggle
      buttons read/write the *same* `GridTuning` fields the M7 dev-panel
      checkboxes already used (`animate_ships`/`show_trajectories`/
      `show_sensor_rings`), not parallel state - but the two DOM surfaces
      don't sync each other's button labels if driven from both places (see
      `hud.rs`'s module doc for why that's an accepted, not overlooked,
      gap). The scanline overlay is pure CSS/DOM (a repeating
      `linear-gradient`, exactly matching `style.css`'s own `.scanlines`
      class) - not a WebGL post-process shader, despite "CRT scanline
      post-pass" sounding like one; the JS reference implements it the same
      way, so there was never a shader pass to port. One real layout bug
      caught and fixed live: the unit-info card and the dev panel both
      wanted the same right-hand screen region and rendered on top of each
      other, illegibly - fixed by pinning the unit card to
      `right: 240px` (clear of the dev panel's own `right: 12px; width:
      220px`) instead of its initial flexbox-driven position. Verified
      live: every toggle (grid/trajectories/ranges/scanlines/animate)
      flips its own button state and the underlying behavior correctly;
      selecting a ship/the station shows the correct name/commander/class
      in the unit card (asteroids show the generic contact); deselecting
      (via Escape, the dev panel's Deselect button, or clicking empty
      space) hides the card again; Reset Camera restores the exact default
      view after an orbit drag; no console errors across extensive
      interaction. Not re-verified live (low-risk, simple field writes):
      the Play/Fast buttons' `speed_multiplier` effect specifically, for
      the same reason M7's own continuous-motion behavior couldn't be
      watched live in this environment (see M7's entry) - the *wiring*
      (button click -> correct `GridTuning` field values) was confirmed by
      reading the code, not by watching motion speed up on screen.

## Notes / gotchas hit so far

- `renderer::webgl::Camera::new` returns `Result<Self, WebglError>` as of the
  user's last `master` merge (input-validation fix, BUG-174) — not `Self`.
  Already fixed here; if re-deriving code from an older read of `camera.rs`,
  remember the `?`.
- The JS `gridTuning.js` defaults (e.g. `dimAlpha: 0.15`) assume the JS
  scene's post-process bloom pass, which doesn't exist in this raw pipeline
  yet (bloom is M8-adjacent polish). Ported numbers that come out "too dim to
  see" in isolation are expected — recalibrate for a bloom-less pipeline
  rather than assuming a porting bug. Already hit twice (ground fill color in
  M0, grid `dimAlpha` in M1).
- `fwidth`-based AA gives a line exactly 1 screen-pixel wide by construction —
  reads as razor-thin/near-invisible at any distance. Added a
  `u_line_width_px` uniform (no JS equivalent) to widen the covered band;
  current default 2.5px works well without softening edges.
- Browser-automation drag/scroll (the `computer` tool) can fail to register
  with `mingl`'s pointer-event handlers because they read `event.screenX/Y`,
  not `clientX/Y` — synthetic events need `screenX`/`screenY` set explicitly
  or the delta is always zero. Not an app bug; just a testing-tool quirk.
- Grid shader intentionally bypasses `Material`/`Renderer` entirely (own VAO,
  own program, own draw call in `main.rs`) — this is the pattern for M3's
  ribbon too, not something to "fix" later.
- Don't call `navigator.clipboard.readText()` from the browser-automation
  `javascript_tool` to verify a copy-to-clipboard button — it triggers a
  clipboard-read permission prompt that hangs the CDP call for the full
  timeout. Verify by checking the button's own text feedback (e.g. it flips
  to "Copied!") instead; that alone proves the `write_text` promise resolved.
- minwebgl's `uniform::upload` on a bare `&[f32]`/`[f32;N]` only accepts
  `N` in `1..=4` — that path is for a single vecN uniform, not an
  arbitrary-length `float[]` array uniform. Uploading `float u_x[16]` from a
  `Vec<f32>` needs each element wrapped as `[f32;1]` (`&[[f32;1]]`) to hit
  the "array of vecN elements, any array length" path instead — the array
  and single-vector paths look almost identical to call but panic
  (`CantUploadUniform`) at runtime if you pick the wrong one. Hit this once
  uploading `u_asteroid_radius` in M3 (`asteroids.rs`'s `glow_candidates` →
  `TacticalGrid::draw`); `vec2`/`vec3` array uniforms (`u_boundary_pts`,
  `u_asteroid_pos`) don't have this trap since `[[f32;2]]`/`[[f32;3]]` were
  never ambiguous with a single vector upload the same way `[f32;N]` is.
- Ray/plane picking (ground click → world point) needs `Mat4::inverse()`
  (`ndarray_cg`'s `d2::mat4x4::general::inverse`) on `view_proj`, then
  unproject NDC `z = -1` and `z = 1` to get near/far world points and
  intersect the ray with `y = 0` — no ready-made unproject/raycast helper
  exists in `mingl`/`renderer` yet. M5 deleted the `main.rs` functions that
  did this (`unproject`/`ray_ground_hit`) since GPU id-buffer picking doesn't
  need a ray at all; M6 re-derived the identical functions for gizmo drag
  math (constraining a drag to the `y = 0` plane), so they're back in
  `main.rs` for good now. Kept here in full too, in case a future milestone
  ever needs the same math in a context where re-deriving from `main.rs`
  isn't convenient:
  ```rust
  fn unproject( inv_view_proj : gl::F32x4x4, ndc_x : f32, ndc_y : f32, ndc_z : f32 ) -> gl::math::F32x3
  {
    let clip = gl::math::F32x4::new( ndc_x, ndc_y, ndc_z, 1.0 );
    let world = inv_view_proj * clip;
    gl::math::F32x3::new( world.x() / world.w(), world.y() / world.w(), world.z() / world.w() )
  }

  fn ray_ground_hit( view_proj : gl::F32x4x4, canvas : &gl::web_sys::HtmlCanvasElement, client_x : f64, client_y : f64 ) -> Option< [ f32; 2 ] >
  {
    let rect = canvas.get_bounding_client_rect();
    let x = ( client_x - rect.left() ) as f32;
    let y = ( client_y - rect.top() ) as f32;
    let w = rect.width() as f32;
    let h = rect.height() as f32;
    if w <= 0.0 || h <= 0.0 { return None; }

    let ndc_x = ( x / w ) * 2.0 - 1.0;
    let ndc_y = 1.0 - ( y / h ) * 2.0;

    let inv = view_proj.inverse()?;
    let near = unproject( inv, ndc_x, ndc_y, -1.0 );
    let far = unproject( inv, ndc_x, ndc_y, 1.0 );
    let dir = far - near;
    if dir.y().abs() < 1e-6 { return None; }
    let t = -near.y() / dir.y();
    if t < 0.0 { return None; }
    let hit = near + dir * t;
    Some( [ hit.x(), hit.z() ] )
  }
  ```
  (object_picking's own approach assumes a camera fixed at the origin, which
  doesn't hold here — that's why this couldn't just be reused as-is.)
- GPU id-buffer picking (M5, `picking.rs`) renders every pickable
  `HullPart`'s `pick_id` into an off-screen `R32I` texture and reads back one
  pixel at the click location, rather than raycasting on the CPU — simpler
  to get right than ray-vs-mesh intersection against the procedural
  box/cylinder/torus/icosphere geometry, and it's exactly what
  `object_picking` already demonstrates. It only re-renders that texture on
  a qualifying click, not every frame, since M4's scene is still fully
  static; **M7's fleet motion will invalidate that assumption** — once ships
  move, the id pass must be re-rendered immediately before every pick (not
  reused from a stale frame), same as `object_picking`'s own `ids_render`
  comment already warns about.
- `tex_storage_2d` (used for the `R32I` id texture) is immutable storage —
  it can't be resized in place. `PickBuffer::resize` in `picking.rs` handles
  a canvas resize by deleting and recreating the texture (and the depth
  renderbuffer, and re-attaching both to the framebuffer) rather than trying
  to reallocate it, and no-ops if the requested size matches what's already
  there (the canvas-resize check in `main.rs`'s render loop calls it on
  every frame, most of which aren't real size changes).
- `PickBuffer::render` sets `gl.viewport` to the id texture's own size while
  drawing into it, since that's usually different from the canvas's. The
  click handler (`setup_selection_click`) is responsible for restoring the
  viewport back to the canvas size afterward — the main render loop only
  calls `gl.viewport` again on an actual resize, not every frame, so a stale
  viewport from a click would otherwise stick until the next resize.
- Flat-shaded low-poly meshes (asteroids) don't need per-face vertex
  duplication in this pipeline — `dFdx`/`dFdy` on the varying world position
  in the fragment shader gives the same "hard face normal" look three.js's
  `flatShading: true` produces, and works with a plain indexed draw. See
  `hull.frag`.
- Starfield looked clustered into a few dense patches instead of spread
  across the scene (M4 follow-up report - three rounds, on the user's own
  machine: Chrome + RTX 2050 mobile, reproducible even in a fresh incognito
  window against a freshly-restarted `trunk serve`, so never a caching
  artifact). What got investigated and RULED OUT along the way, so it
  doesn't get re-litigated later:
  - CPU-side RNG/box math: confirmed correct via a temporary `gl::info!`
    dump of the generated positions (`min≈[-599,-300,-599]
    max≈[600,299,599]`, individual samples nicely spread, every time).
  - "WebGL doesn't clip GL_POINTS behind the camera (w <= 0)": plausible-
    sounding, informed a temporary shader fix, but a live CPU-vs-GPU
    cross-check (CPU counted 615/2000 stars behind the camera at a moment
    the user reported clustering; a shader variant that colored exactly
    those points bright red instead of clipping them rendered *zero* red
    pixels) proved the user's own GPU already discards them correctly on
    its own. This was a real, common occurrence (~30% of stars at a typical
    view angle), just never the actual bug.
  - What most likely mattered: `gl_PointSize` distance attenuation (three.js
    `sizeAttenuation: true`, ported here as `gl_PointSize = clamp(700.0 /
    gl_Position.w, 1.0, 4.0)` in `starfield.vert`) — without it every star
    reads at the same size/prominence regardless of depth, which visually
    exaggerates the perspective clustering inherent to a box this large
    (far more points lie far from the camera than near it). Combined with
    at least one round that really was a stale build despite the user's own
    cache-busting attempts (never root-caused which specific rebuild fixed
    it, since several changes landed close together) - if this regresses
    again, re-add the diagnostics described below rather than assuming
    either explanation without re-checking.
  - Diagnostic technique worth reusing for any future "looks wrong but I
    can't repro it" report: a live on-screen debug HUD (plain `<div>`,
    updated via `set_text_content` each frame - see the git history around
    this note for the exact snippet if needed) showing camera state and a
    CPU-computed cross-check number, plus a shader variant that visually
    marks the flagged elements (e.g. bright red) instead of silently
    discarding them. Directly answers "is it the data or the rendering"
    without needing the user to dig through devtools.

- **Gizmo handle lost the depth test in the id pass** (M6 bug, caught during
  live verification, not shipped). Symptom: clicking dead-center on the
  visible translate cross still picked the *object's* id, not the gizmo's -
  dragging never started, and the click instead fell through to camera-orbit
  behavior (the drag threshold check never fired since `drag_state` stayed
  `None`). Root cause: `Gizmo::draw` (the *visible* pass) explicitly disables
  depth test so the handle stays visible/clickable through its object, but
  `PickBuffer::render` (the *id* pass) drew the gizmo part through the same
  loop as every other part, with depth test at its normal always-on setting
  - the object's own hull geometry usually has *some* geometry closer to the
  camera than the handle's paper-thin plane, so the object's fragments won
  the depth test and overwrote the gizmo's id in the id buffer, even where
  the handle was clearly the topmost thing on screen. Fixed by giving
  `PickBuffer::render` a separate `gizmo_part : Option<&HullPart>` parameter,
  drawn last with depth test explicitly off - matching the visible pass's
  own choice, not just visually consistent with it. **The general lesson**:
  any object meant to render "always on top" (depth test off) needs that
  same treatment in *every* pass that determines what the user is
  interacting with, not just the one that determines what they see - a
  future third overlay type would need the same fix applied again, this
  isn't something `PickBuffer` enforces generically.
- **Browser-automation clicks are unreliable on the very first interaction
  after a fresh page load** (M6 verification quirk, not an app bug) - a
  `computer` tool `left_click` on the very first attempt after `navigate`
  sometimes dispatches nothing at all (confirmed via a manually-injected
  `window.addEventListener` probe: zero events observed, not even
  `mousedown`/`click`, for that first click). A second click at the exact
  same coordinate always works normally afterward, and once the page has
  processed any other interaction (even an unrelated `javascript_tool` eval)
  the very next click is reliable too. Root-caused as a CDP/tab-focus
  timing quirk external to the app, not a real bug: was already ruled out as
  an app issue by directly probing with plain DOM listeners rather than
  trusting the app's own visible state. If a fresh-reload test looks like
  "nothing happened," retry the same click before suspecting the app.
- Windows `trunk serve`'s known snippets-directory move-lock flakiness (see
  the M0-era note earlier in this section) got *more* frequent during M6's
  rapid edit-reload cycles - confirmed harmless again each time by checking
  the actually-served `dist/*.js`/`*.wasm` file timestamps and/or `curl`ing
  the served HTML directly rather than trusting a single browser reload,
  since a reload landing exactly on a failed build serves trunk's own
  error-overlay page (which looks nothing like a crash, easy to misread as
  "the app broke"). When in doubt, kill trunk, `rm -rf dist`, and restart
  clean on a fresh port rather than chasing a stale server state.
- **Wrong sign in the Catmull-Rom cubic term** (M7 bug, caught by a unit
  test before ever reaching the browser). `spline.rs`'s `catmull_rom_segment`
  transcribed the standard basis-matrix formula's cubic coefficient as
  `-p0 - 3p1 + 3p2 - p3` instead of the correct `-p0 + 3p1 - 3p2 + p3` (sign
  flipped on the `p1`/`p2`/`p3` terms). Visually this was *not* obviously
  wrong - mid-path curve evaluation still produced a smooth-looking (if
  geometrically incorrect) line, which is exactly why it's worth recording:
  a plausible-but-wrong closed-form formula can pass a "does this look like
  a curve" visual check while still being mathematically wrong. It only
  became unambiguous at a path boundary: `point_at_progress(path, 1.0)` on
  a 2-waypoint path returned `[15.0, 15.0]` instead of the exact second
  waypoint `[5.0, 5.0]` - `cargo test --bin falling_frontier spline` catches
  this in under a second, with zero WebGL/browser involved. **The general
  lesson**: for closed-form math ported from a formula (not from another
  codebase's working implementation), a handful of boundary-value unit
  tests (`t=0` returns the first point exactly, `t=1` returns the last
  point exactly, a degenerate/minimal input doesn't panic) catch
  transcription errors that visual smoke-testing reliably misses, and cost
  far less than another round of `trunk serve`/browser automation. Six
  tests now live in `spline.rs`'s own `#[cfg(test)] mod tests` - run them
  with `cargo test --bin falling_frontier` (this crate has no `[lib]`
  target, so `--lib` doesn't work; `--bin falling_frontier` does, and runs
  natively - no wasm target/browser needed since `spline.rs` only touches
  plain vector math, no GL/DOM).
- **Browser-automation cannot observe continuous per-frame animation in
  this environment** (M7 finding, not an app bug - distinct from the
  already-documented "first click after reload is flaky" quirk). Toggling
  "Animate Ships" produces one correct, immediate reorientation (ships snap
  to face their path's initial tangent - proof `advance()` ran at least
  once), but position visibly stops there: a direct probe
  (`window.requestAnimationFrame` patched to count calls) recorded **zero**
  calls over 23 real seconds, and `document.hidden` read `true` the whole
  time despite `document.hasFocus()` reading `true`. Chrome fully suspends
  `requestAnimationFrame` for a page it considers not actually visible
  (distinct from ordinary background-tab throttling to ~1fps - this was a
  hard stop at 0), and neither repeated clicks nor waiting longer changed
  that in this session. Since `gl::exec_loop::run`'s render loop - and so
  every per-frame mutation, not just fleet motion - depends entirely on
  `requestAnimationFrame`, this means **any future milestone's "watch it
  move/animate/pulse over time" behavior cannot be verified by screenshot-
  over-time in this environment** the way M1-M6's static/one-shot-drag
  visuals could be. When that's needed again: verify the underlying math
  with unit tests instead (see the bug note above - this is exactly how
  M7's real bug was actually caught, not despite this limitation but
  because of leaning on tests instead of fighting it), and treat any one
  visible frame (e.g. right after a checkbox toggle) as confirmation the
  *wiring* is connected, not as confirmation of the ongoing behavior over
  time.
- **Two independent DOM overlays claiming the same screen region** (M8 bug,
  caught live). `hud.rs`'s unit-info card was originally laid out via
  flexbox `space-between` inside the HUD's full-width root, which put it
  flush against the right edge of the viewport - exactly where
  `debug/grid_tuning_panel.rs`'s dev panel already lives
  (`position: fixed; right: 12px`). Both panels rendered, stacked on top of
  each other, illegible. Two separately-built DOM trees with no shared
  layout system don't know about each other's space unless something makes
  them - fixed by pulling the unit card out of flex flow entirely
  (`position: fixed; right: 240px`, clearing the dev panel's own 220px
  width + margin) rather than trying to coordinate the two panels' layout
  systems. Worth remembering if a third floating panel ever gets added:
  fixed-position DOM overlays in this crate don't self-arrange, whoever
  adds the next one needs to explicitly avoid the existing ones' rectangles.

## Verification pattern used so far

`trunk serve --port <N>` in background → `mcp__claude-in-chrome` tools to
navigate/screenshot/drag → kill trunk + `rm -rf dist` when done. Always check
`cargo check` and `cargo clippy` on `--target wasm32-unknown-unknown` before
declaring a milestone done. **As of M7**: also run `cargo test --bin
falling_frontier` for any new pure-math module (no wasm target needed) -
catches transcription bugs visual testing can miss, and is unaffected by
the `document.hidden`/`requestAnimationFrame` limitation noted above.

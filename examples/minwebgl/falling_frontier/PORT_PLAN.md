# Falling Frontier → cgtools port: working plan

Source of truth for progress on this port. Update this file as milestones land —
it's meant to survive context resets, so write for a reader who has none of the
current conversation.

## Resume here

M0-M5 are done, verified live in-browser, and **committed** (see their
checklist entries below for what/how):
- `851dd9df` on `space-game-demo` — M0-M3 ("feat: add Falling Frontier
  tactical grid, dev panel, and view-zone ribbon")
- `6c71a5c8` on `space-game-demo` — M4 ("feat: add Falling Frontier ships,
  station, and starfield (M4)")
- `0a8434ea` on `space-game-demo` — M5 ("feat: add Falling Frontier real
  object picking/selection (M5)")

All with no `Co-Authored-By` trailer, per the standing repo rule (see memory
`feedback_commit_trailers`). Nothing else in this crate is uncommitted as of
this note. `examples/minwebgl/falling_frontier/Untitled.png` is an untracked
debug screenshot the user pasted in during the M4 starfield investigation
(see Notes section below) — left untracked on purpose, safe to delete once
no longer needed, not part of the deliverable.

**Next task: M6** — transform gizmo (translate XZ / rotate Y, G/R/Escape).
No gizmo exists anywhere in cgtools. `main.rs`'s `selected_id : Rc<Cell<
Option<i32>>>` (M5) is the thing to attach the gizmo to — resolve it to a
`&HullPart`'s (or, for ships, every `HullPart` sharing that ship's
`pick_id`) `model` matrix the same way the render loop's highlight check
already does (`Some(part.pick_id) == selected`). Dragging will need the
ray/plane ground-unprojection math M5 deleted from `main.rs`
(`unproject`/`ray_ground_hit`, removed because GPU id-buffer picking doesn't
need a ray at all) — it's preserved in full below under "Notes / gotchas",
re-derive it there rather than trying to recover the deleted functions from
git history. Translate mode constrains the drag to the `y = 0` plane (same
plane M3's ground click used to hit-test against), rotate mode only needs
the Y angle. `station.rs`/`ships.rs`/`asteroids.rs` currently bake each
part's `model` matrix once at construction and never touch it again —
dragging will need those matrices to become mutable per-frame state instead
(probably: store a per-object base transform + live translation/rotation
override, recompute `model` each frame, rather than mutating the baked
matrix in place, so the original spec position is never lost if the user
wants a "reset" affordance later).

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
- [ ] **M6** — transform gizmo (translate XZ / rotate Y, G/R/Escape, attach
      idle-animation suppression). No gizmo exists anywhere in cgtools.
- [ ] **M7** — fleet motion + trajectories: Catmull-Rom spline (new, on top of
      existing Hermite/Bezier easing in `module/helper/animation`),
      `line_tools`-based trajectory ribbon + waypoint rings + dashed sensor
      ring. `line_tools` is the right tool HERE (a handful of rings/paths) —
      it was deliberately NOT used for the base grid in M1 (would mean
      hundreds of individual line meshes per frame vs. one shader draw call).
- [ ] **M8** — HUD/DOM overlay + polish (status bar, unit-info card, toolbar,
      CRT scanline post-pass). Lowest priority, pure UI chrome.

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
  exists in `mingl`/`renderer` yet. **M5 deleted the `main.rs` functions that
  did this** (`unproject`/`ray_ground_hit`) since GPU id-buffer picking
  doesn't need a ray at all, but **M6's gizmo dragging will need the same
  math again** (constraining a drag to the `y = 0` plane), so it's preserved
  here rather than left to bit-rot in git history:
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

## Verification pattern used so far

`trunk serve --port <N>` in background → `mcp__claude-in-chrome` tools to
navigate/screenshot/drag → kill trunk + `rm -rf dist` when done. Always check
`cargo check` and `cargo clippy` on `--target wasm32-unknown-unknown` before
declaring a milestone done.

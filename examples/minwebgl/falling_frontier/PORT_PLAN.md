# Falling Frontier → cgtools port: working plan

Source of truth for progress on this port. Update this file as milestones land —
it's meant to survive context resets, so write for a reader who has none of the
current conversation.

## Resume here

M0, M1, M2, M3 are done and verified live in-browser (see their checklist
entries below for what/how). **Nothing is committed to git yet** — working
tree has two untracked additions: `examples/minwebgl/falling_frontier/`
(this whole crate) and `research/falling_frontier_cgtools_audit.md`. Ask the
user whether to commit before starting M4, or keep going uncommitted — this
has been asked before without an answer, so don't assume either way.

**Next task: M4** — full static scene content: ship hulls, station,
starfield. No procedural primitive generators for boxes/cylinders/cones exist
in cgtools yet (`primitive_generation` only has curve/plane/contour/text
generators) — will likely need new additions there, consistent with the
standing "extending cgtools crates is the point of the port" instruction.
Read `examples/threejs/falling_frontier/src/world/` for the ship/station/
starfield JS references before starting (not yet read in depth this session
— `asteroidBelt.js` and `tacticalGrid.js` are the only two read so far).
Also worth doing before/alongside M4: swap `asteroids.rs`'s simplified
icosahedron placeholder for something closer to the JS dodecahedron look if
it reads as too rough once real ships are in the scene (see M3 notes below —
this was a deliberate simplification, not an oversight).

M5 (real picking, replacing M3's ground-click stand-in) and M6 (gizmo) both
still need the ship/station meshes from M4 to be meaningful, so M4 is the
right next step even though M5/M6 were "next" in the original risk-first
ordering rationale — the ribbon/grid risk (M1–M3) is now retired.

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
- [ ] **M4** — full static scene content: ship hulls, station, starfield.
      No procedural primitive generators (box/cylinder/cone compositing)
      exist in cgtools yet — `primitive_generation` only has
      curve/plane/contour generators. Likely a new addition here.
- [ ] **M5** — real object picking/selection, replacing M3's ground-click
      stand-in. Extend `examples/minwebgl/object_picking`'s GPU-ID-buffer
      pattern.
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
  exists in `mingl`/`renderer` yet, this was written from scratch in
  `main.rs` (`unproject`/`ray_ground_hit`). Worth promoting to a shared
  crate if M5's real picking needs the same math (object_picking's own
  approach assumes a camera fixed at the origin, which doesn't hold here).
- Flat-shaded low-poly meshes (asteroids) don't need per-face vertex
  duplication in this pipeline — `dFdx`/`dFdy` on the varying world position
  in the fragment shader gives the same "hard face normal" look three.js's
  `flatShading: true` produces, and works with a plain indexed draw. See
  `asteroid.frag`.

## Verification pattern used so far

`trunk serve --port <N>` in background → `mcp__claude-in-chrome` tools to
navigate/screenshot/drag → kill trunk + `rm -rf dist` when done. Always check
`cargo check` and `cargo clippy` on `--target wasm32-unknown-unknown` before
declaring a milestone done.

//! Live-tunable state for the tactical grid shader, ported from
//! `examples/threejs/falling_frontier/src/debug/gridTuning.js`. Scoped to
//! only the uniforms `TacticalGrid` (M1) actually wires — extend this
//! struct (not a second one) when M3 adds the ribbon/glow uniforms.

/// Fade curve shapes shared by the camera-distance fade (and, later, the
/// inside/ribbon fade). Value is what the shader's `u_camera_fade_mode`
/// uniform expects.
pub const FADE_CURVES : [ ( f32, &str ); 4 ] =
[
  ( 0.0, "Linear" ),
  ( 1.0, "Smoothstep" ),
  ( 2.0, "Exponential" ),
  ( 3.0, "Exponential^2" ),
];

#[ derive( Clone, Copy ) ]
pub struct GridTuning
{
  pub line_color : [ f32; 3 ],
  pub dim_alpha : f32,
  pub cell_size : f32,
  pub line_width_px : f32,
  pub camera_fade_start : f32,
  pub camera_fade_end : f32,
  pub camera_fade_mode : f32,
  pub camera_fade_gamma : f32,

  // M3: view-zone ribbon + asteroid glow, ported from the JS `gridTuning`
  // object's remaining fields (everything above this point was already
  // there for M1). `bright_alpha` is bumped above the JS default (0.4 → 0.85)
  // for the same no-bloom reason `dim_alpha` was in M1 — see PORT_PLAN.md.
  pub bright_alpha : f32,
  pub ribbon_color_core : [ f32; 3 ],
  pub ribbon_color_edge : [ f32; 3 ],
  pub ribbon_width_outer : f32,
  pub ribbon_width_inner : f32,
  pub ribbon_gap : f32,
  pub ribbon_opacity : f32,
  pub inside_fade_width : f32,
  pub inside_fade_mode : f32,
  pub inside_fade_gamma : f32,
  pub asteroid_glow_alpha : f32,
  pub asteroid_glow_width : f32,
  pub asteroid_glow_mode : f32,
  pub asteroid_glow_gamma : f32,

  // Every ship shares one view radius in this scene (`fleet.js`'s
  // `FLEET_VIEW_RADIUS`), so this doubles as both the dev-tuning default and
  // the JS reference's `gridTuning.viewRadiusOverride` - no separate
  // per-ship value to look up.
  pub view_radius : f32,

  // M7: fleet motion + trajectory/sensor-ring visibility. `animate_ships`
  // defaults to `false`, matching the JS reference's own
  // `playbackState.isAnimating: false` ("off by default while the static
  // layout is being blocked out with the transform gizmo" - see
  // examples/threejs/falling_frontier/src/state.js). `show_trajectories`/
  // `show_sensor_rings` default to `false` too, matching `main.js`'s
  // `groups.trajectory.visible = false; groups.sensorRing.visible = false;`.
  pub animate_ships : bool,
  pub show_trajectories : bool,
  pub show_sensor_rings : bool,

  // M8: HUD toolbar state. `show_grid` defaults to `true` (JS's own
  // `toggle-grid` button starts `active`/`[ON]`); `speed_multiplier` scales
  // `animate_ships`'s per-frame progress step - the HUD's Play/Fast buttons
  // set it to `1.0`/`2.5` (matching `playbackState.shipSpeedMultiplier` in
  // the JS reference), Pause leaves it alone and just clears `animate_ships`.
  pub show_grid : bool,
  pub speed_multiplier : f32,

  // Directional light + shadow-map controls for `hull.rs`'s material
  // (asteroids/ships/station) - not part of the JS reference's own dev
  // panel (`gridTuningPanel.js` never exposed lighting), added per explicit
  // request once the hull material grew a real directional-light model with
  // shadow mapping instead of the flat ambient+diffuse it started with.
  // Azimuth/elevation (not a raw direction vector) since that's what's
  // actually pleasant to drag on a slider - `main.rs` converts to a
  // direction each frame.
  pub light_azimuth : f32,
  pub light_elevation : f32,
  pub light_color : [ f32; 3 ],
  pub light_intensity : f32,
  // `Light::size()` ("controls shadow softness" per its own doc comment,
  // `module/helper/renderer/src/webgl/shadow.rs:342`) - was a hardcoded
  // `1.0` literal at the `Light::new` call site in `main.rs` until this
  // field exposed it. Range mirrors this same renderer's own spot-light
  // precedent (`shadow.rs:454`: `light_size` computed in `0.01..=1.7`).
  pub light_size : f32,
  pub shadows_enabled : bool,

  // Render-layer isolation switches - one per distinct draw call/pass in
  // `main.rs`'s frame closure, so any combination of scene layers can be
  // shown alone or hidden alone (e.g. "only the grid", "everything but
  // asteroids"). `lighting_enabled` is deliberately separate from
  // `shadows_enabled`: the former drops `hull.frag` to a flat unlit
  // `u_color` (see hull.frag's `u_lighting_enabled` branch), the latter only
  // gates the shadow-map sample within the normal lit path. `show_asteroids`/
  // `show_ships`/`show_station` also gate that object's contribution to the
  // shadow-caster pass, not just its own visible draw - a hidden object
  // shouldn't still be casting a shadow onto the rest of the scene.
  pub show_background : bool,
  pub show_starfield : bool,
  pub show_asteroids : bool,
  pub show_ships : bool,
  pub show_station : bool,
  pub show_view_ribbon : bool,
  pub show_gizmo : bool,
  pub lighting_enabled : bool,
  /// CRT scanline overlay - pure DOM/CSS effect (see `hud.rs`'s `ff-scanlines`
  /// element), not a WebGL draw call, but tracked here anyway so it lives in
  /// the same single Render Layers menu as every other switch instead of
  /// needing its own separate on/off surface.
  pub show_scanlines : bool,
}

impl Default for GridTuning
{
  fn default() -> Self
  {
    Self
    {
      line_color : [ 0.0, 0.847, 0.965 ], // COLORS.gridCyan (0x00d8f6)
      dim_alpha : 0.21,
      cell_size : 10.0,
      line_width_px : 1.0,
      camera_fade_start : 0.0,
      camera_fade_end : 860.0,
      camera_fade_mode : 2.0,
      camera_fade_gamma : 1.25,

      bright_alpha : 0.5,
      ribbon_color_core : [ 0.729, 0.925, 0.996 ], // #baecfe
      ribbon_color_edge : [ 0.0, 0.847, 0.965 ],   // #00d8f6
      ribbon_width_outer : 0.8,
      ribbon_width_inner : 1.1,
      ribbon_gap : 0.3,
      ribbon_opacity : 1.0,
      inside_fade_width : 1.6,
      inside_fade_mode : 2.0,
      inside_fade_gamma : 0.5,
      asteroid_glow_alpha : 1.0,
      asteroid_glow_width : 6.1,
      asteroid_glow_mode : 3.0,
      asteroid_glow_gamma : 0.7,

      view_radius : 160.0,

      animate_ships : false,
      show_trajectories : false,
      show_sensor_rings : false,

      show_grid : true,
      speed_multiplier : 1.0,

      light_azimuth : 276.0,
      light_elevation : 31.0,
      light_color : [ 1.0, 0.933, 0.867 ], // 0xffeedd, matches world.js's own sunLight color
      light_intensity : 1.85,
      light_size : 1.0,
      shadows_enabled : true,

      show_background : true,
      show_starfield : true,
      show_asteroids : true,
      show_ships : true,
      show_station : true,
      show_view_ribbon : true,
      show_gizmo : true,
      lighting_enabled : true,
      show_scanlines : false,
    }
  }
}

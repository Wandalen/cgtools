//! Live-tunable state shared by the dev panels, the HUD and the frame loop.
//! Started as the tactical grid shader's uniforms, ported from the three.js
//! original's grid tuning object, and has since grown the view-zone ribbon
//! and asteroid glow (M3), fleet playback (M7/M8) and the directional light.
//! The render-layer switches live in their own `RenderLayers`, held here as
//! `layers`, so one `Rc< RefCell< GridTuning > >` still carries everything.

use super::render_layers::RenderLayers;

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

/// Every live-tunable value the demo has; see the module doc for what each
/// group is and `RenderLayers` for the visibility switches.
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

  // M7: fleet motion. `animate_ships` defaults to `false`, matching the
  // three.js original, which also started with ship animation off while the
  // static layout was being blocked out with the transform gizmo.
  pub animate_ships : bool,

  // M8: `speed_multiplier` scales `animate_ships`'s per-frame progress step
  // - the HUD's Play/Fast buttons set it to `1.0`/`2.5` (matching
  // `playbackState.shipSpeedMultiplier` in the JS reference), Pause leaves
  // it alone and just clears `animate_ships`.
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
  // Shadow softness: `hull.frag`'s `u_light_size`, which scales the 3x3 PCF
  // tap spacing in shadow-map texels (0 = hard edge, 1 = the original
  // one-texel spacing). Also passed to `Light::new`, whose own `size()` only
  // the renderer's `ShadowBaker` reads - this example doesn't use it, so the
  // uniform is what makes the slider visible.
  pub light_size : f32,

  /// Which scene layers the frame loop draws - the Render Layers panel's
  /// switches.
  pub layers : RenderLayers,
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
      speed_multiplier : 1.0,

      light_azimuth : 276.0,
      light_elevation : 31.0,
      light_color : [ 1.0, 0.933, 0.867 ], // 0xffeedd, matches world.js's own sunLight color
      light_intensity : 1.85,
      light_size : 1.0,

      layers : RenderLayers::default(),
    }
  }
}

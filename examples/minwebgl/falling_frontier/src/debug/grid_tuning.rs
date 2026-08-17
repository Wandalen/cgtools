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

  // Stand-in for "selected unit's view radius" (real unit selection is M5) —
  // the ground-click focus point in `main.rs` always uses this radius.
  pub view_radius : f32,
}

impl Default for GridTuning
{
  fn default() -> Self
  {
    Self
    {
      line_color : [ 0.0, 0.847, 0.965 ], // COLORS.gridCyan (0x00d8f6)
      dim_alpha : 0.45,
      cell_size : 10.0,
      line_width_px : 2.5,
      camera_fade_start : 0.0,
      camera_fade_end : 950.0,
      camera_fade_mode : 2.0,
      camera_fade_gamma : 0.95,

      bright_alpha : 0.85,
      ribbon_color_core : [ 0.729, 0.925, 0.996 ], // #baecfe
      ribbon_color_edge : [ 0.0, 0.847, 0.965 ],   // #00d8f6
      ribbon_width_outer : 0.8,
      ribbon_width_inner : 1.1,
      ribbon_gap : 0.3,
      ribbon_opacity : 1.0,
      inside_fade_width : 1.1,
      inside_fade_mode : 2.0,
      inside_fade_gamma : 0.95,
      asteroid_glow_alpha : 1.0,
      asteroid_glow_width : 4.2,
      asteroid_glow_mode : 1.0,
      asteroid_glow_gamma : 1.05,

      view_radius : 160.0,
    }
  }
}

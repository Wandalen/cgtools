//! Native tests for the `scene` module — `Color`'s uniform-layout conversion
//! and full `scene.rhai` round-trip parsing via `SceneConfig::load()`.
//!
//! Relocated from `src/scene.rs`, per the all-tests-in-tests/ convention.

use orrery_webgpu::scene::
{
  Background, Color, Effects, Grid, NebulaBand, Node, OrbitRing, SceneConfig,
  StarLayer, SunCorona, SunDisc,
};

/// `main.rs`'s wasm32-gated `run()` is the only caller of `to_array()` on
/// the wasm32 target; on native, this test is what keeps it from being
/// dead code, and isolates the conversion itself from schema parsing.
///
/// `to_array()`'s `as f32` narrowing is an IEEE-754 basic conversion operation —
/// specified as correctly-rounded on every target, including wasm32's `f32.demote_f64`
/// (unlike a libm transcendental call, whose rounding is implementation-defined). Verified
/// empirically that `0.1_f64/0.2_f64/0.3_f64 as f32` land on the exact same bit pattern as
/// the `f32` literals compared against, so no double-rounding hazard applies here.
#[ allow( clippy::float_cmp, reason = "to_array()'s `as f32` narrowing is an IEEE-754 basic conversion, correctly-rounded on every target including wasm32's f32.demote_f64; verified empirically that 0.1_f64/0.2_f64/0.3_f64 as f32 land on the exact same bit pattern as the f32 literals compared against, so no double-rounding hazard applies here" ) ]
#[ test ]
fn color_to_array_appends_opaque_alpha()
{
  assert_eq!( Color( 0.1, 0.2, 0.3 ).to_array(), [ 0.1, 0.2, 0.3, 1.0 ] );
}

/// Asserts every field `scene.rhai` declares round-trips through
/// `SceneConfig::load()` correctly — one exhaustive struct-level
/// comparison (via derived `PartialEq`) rather than a spot check, so
/// every field of every list entry is exercised on every target,
/// including native, where `main.rs`'s wasm32-gated `run()` — the only
/// other consumer — never compiles in and can't do it instead.
//
// Exact comparison is intentional: every value here is a literal parsed
// straight out of scene.rhai with no arithmetic in between, so bit-exact
// round-trip fidelity is exactly what this test means to check.
#[ test ]
fn scene_rhai_parses_and_matches_known_values()
{
  let scene = SceneConfig::load();

  let expected = SceneConfig
  {
    background : Background
    {
      top : Color( 0.0196, 0.0549, 0.0941 ),
      bottom : Color( 0.0549, 0.1490, 0.2392 ),
    },
    nebula_bands : vec!
    [
      NebulaBand { color : Color( 0.0706, 0.2000, 0.2902 ), opacity : 0.45, center : 0.5,  thickness : 0.30, noise_scale : 1.0, drift_speed :  0.020 },
      NebulaBand { color : Color( 0.2510, 0.1216, 0.3608 ), opacity : 0.28, center : 0.22, thickness : 0.16, noise_scale : 1.6, drift_speed :  0.035 },
      NebulaBand { color : Color( 0.3608, 0.1490, 0.1804 ), opacity : 0.22, center : 0.80, thickness : 0.14, noise_scale : 1.3, drift_speed : -0.025 },
    ],
    star_layers : vec!
    [
      StarLayer { color : Color( 0.6275, 0.8980, 1.0 ), intensity : 0.6, density : 9.0, size : 0.06, twinkle_speed : 1.5 },
      StarLayer { color : Color( 0.8353, 0.9569, 1.0 ), intensity : 0.9, density : 4.0, size : 0.03, twinkle_speed : 0.8 },
    ],
    grid : Grid { color : Color( 0.3137, 0.5490, 0.7451 ), opacity : 0.18, line_width : 1.0, glow : 0.15 },
    sun_corona : SunCorona
    {
      inner : Color( 1.0, 0.8941, 0.4392 ), mid : Color( 1.0, 0.6824, 0.1020 ), outer : Color( 1.0, 0.2314, 0.0 ),
      inner_radius : 0.08, mid_radius : 0.15, outer_radius : 0.25,
      flicker_amplitude : 0.08, flicker_speed : 2.4,
    },
    sun_disc : SunDisc
    {
      dark : Color( 1.0, 0.4157, 0.0 ), mid : Color( 1.0, 0.8941, 0.4392 ), bright : Color( 1.0, 1.0, 1.0 ),
      base_radius : 0.075, pulsate_amplitude : 0.015, pulsate_speed : 1.1, granulation_scale : 1.0,
    },
    orbit_rings : vec!
    [
      OrbitRing { color : Color( 0.3922, 0.8235, 1.0 ), radius : 0.425, glow : 0.35, stroke_width : 1.00, pulse_speed : 0.60 },
      OrbitRing { color : Color( 0.6000, 0.7000, 1.0 ), radius : 0.600, glow : 0.28, stroke_width : 0.85, pulse_speed : 0.45 },
      OrbitRing { color : Color( 0.8500, 0.6000, 1.0 ), radius : 0.780, glow : 0.20, stroke_width : 0.70, pulse_speed : 0.30 },
    ],
    nodes : vec!
    [
      Node { color : Color( 1.00, 0.75, 0.45 ), size : 0.014, radius : 0.425, speed :  0.50, phase : 0.0 },
      Node { color : Color( 0.55, 0.85, 1.00 ), size : 0.011, radius : 0.425, speed : -0.35, phase : 3.4 },
      Node { color : Color( 0.60, 0.90, 0.60 ), size : 0.016, radius : 0.600, speed :  0.32, phase : 1.1 },
      Node { color : Color( 0.90, 0.55, 0.90 ), size : 0.010, radius : 0.600, speed : -0.22, phase : 4.6 },
      Node { color : Color( 1.00, 0.95, 0.60 ), size : 0.019, radius : 0.780, speed :  0.20, phase : 2.0 },
      Node { color : Color( 0.75, 0.80, 1.00 ), size : 0.012, radius : 0.780, speed : -0.15, phase : 5.2 },
    ],
    effects : Effects { vignette_strength : 0.35, vignette_radius : 0.55, glow_intensity : 1.1, scanline_intensity : 0.05 },
  };

  assert_eq!( scene, expected );
}

//! Deserializable scene configuration for the sun/grid HUD diagram, loaded
//! from `scene.rhai` at compile time via `include_str!` and evaluated
//! through `scene_script::build_engine()`. Kept free of any wasm/WebGPU
//! dependency so parsing can be unit-tested on the native target, unlike
//! the rest of this crate.

use serde::Deserialize;

/// An RGB color, deserialized from a Rhai `[ r, g, b ]` array. Elements are
/// `f64` — Rhai's own `FLOAT` type — rather than `f32`: `rhai`'s serde
/// bridge matches the dynamic value's actual type exactly and does not
/// narrow, so an `f32` field here would fail to deserialize at all. See
/// [`to_array`](Self::to_array) for the narrowing cast back to `f32`, and
/// `scene_script::vector_binding` for the same crossing-the-boundary
/// pattern applied to `F32x2`.
#[ derive( Debug, Clone, Copy, PartialEq, Deserialize ) ]
pub struct Color( pub f64, pub f64, pub f64 );

impl Color
{
  /// Narrows to `[ r, g, b, 1.0 ]` — the `vec4f`-packed layout `scene.wgsl`'s
  /// `Uniforms` struct uses for every color field, sidestepping WGSL's
  /// vec3-aligns-to-16 padding rules entirely.
  pub fn to_array( self ) -> [ f32; 4 ]
  {
    [ self.0 as f32, self.1 as f32, self.2 as f32, 1.0 ]
  }
}

#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct Background
{
  pub top : Color,
  pub bottom : Color,
}

/// One fog layer drifting across the vertical middle of the frame —
/// `scene.rhai` declares exactly [`NEBULA_BAND_COUNT`] of these.
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct NebulaBand
{
  pub color : Color,
  pub opacity : f64,
  pub center : f64,
  pub thickness : f64,
  pub noise_scale : f64,
  pub drift_speed : f64,
}

/// One background star field: a hashed point per grid cell at `density`,
/// twinkling at `twinkle_speed` — `scene.rhai` declares exactly
/// [`STAR_LAYER_COUNT`] of these.
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct StarLayer
{
  pub color : Color,
  pub intensity : f64,
  pub density : f64,
  pub size : f64,
  pub twinkle_speed : f64,
}

#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct Grid
{
  pub color : Color,
  pub opacity : f64,
  pub line_width : f64,
  pub glow : f64,
}

#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct SunCorona
{
  pub inner : Color,
  pub mid : Color,
  pub outer : Color,
  pub inner_radius : f64,
  pub mid_radius : f64,
  pub outer_radius : f64,
  pub flicker_amplitude : f64,
  pub flicker_speed : f64,
}

#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct SunDisc
{
  pub dark : Color,
  pub mid : Color,
  pub bright : Color,
  pub base_radius : f64,
  pub pulsate_amplitude : f64,
  pub pulsate_speed : f64,
  pub granulation_scale : f64,
}

/// One concentric orbit rail — `scene.rhai` declares exactly
/// [`ORBIT_RING_COUNT`] of these.
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct OrbitRing
{
  pub color : Color,
  pub radius : f64,
  pub glow : f64,
  pub stroke_width : f64,
  pub pulse_speed : f64,
}

/// One authored planet/moon orbiting at a fixed `radius` with angular
/// `speed` (sign gives direction) and starting `phase` — independent of
/// `scene.wgsl`'s keyboard-driven procedural nodes, which keep working
/// unchanged. `scene.rhai` declares exactly [`NODE_COUNT`] of these.
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct Node
{
  pub color : Color,
  pub size : f64,
  pub radius : f64,
  pub speed : f64,
  pub phase : f64,
}

/// Cross-cutting single-pass effects applied after every primitive above is
/// composited. No offscreen texture exists to sample at an offset, so
/// effects are limited to what a single analytic pass can compute directly
/// (no true multi-tap effect like chromatic aberration).
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct Effects
{
  pub vignette_strength : f64,
  pub vignette_radius : f64,
  pub glow_intensity : f64,
  pub scanline_intensity : f64,
}

/// List lengths `scene.rhai` must declare exactly, mirrored by
/// `scene.wgsl`'s `array<vec4f, N>` uniform fields — a WGSL `uniform`
/// binding's arrays must be a compile-time fixed size, so there is no
/// runtime element count to fall back on.
pub const NEBULA_BAND_COUNT : usize = 3;
pub const STAR_LAYER_COUNT : usize = 2;
pub const ORBIT_RING_COUNT : usize = 3;
pub const NODE_COUNT : usize = 6;

/// The HUD diagram's full scene description — everything about it that
/// isn't already live-adjustable via the keyboard (seed, node count, grid
/// density stay runtime-interactive; see `Params` in `main.rs`).
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct SceneConfig
{
  pub background : Background,
  pub nebula_bands : Vec< NebulaBand >,
  pub star_layers : Vec< StarLayer >,
  pub grid : Grid,
  pub sun_corona : SunCorona,
  pub sun_disc : SunDisc,
  pub orbit_rings : Vec< OrbitRing >,
  pub nodes : Vec< Node >,
  pub effects : Effects,
}

impl SceneConfig
{
  const SCRIPT : &'static str = include_str!( "../scene.rhai" );

  /// Evaluates the bundled `scene.rhai` and extracts a `SceneConfig` from
  /// its returned value via `rhai`'s serde bridge. Panics on a malformed
  /// script, a returned shape that doesn't match `SceneConfig`, or a list
  /// whose length doesn't match its fixed `scene.wgsl` uniform array size —
  /// the script is compiled into the binary by this crate itself, not
  /// supplied by an end user, so a failure here is a build-time authoring
  /// mistake that should fail loudly and immediately rather than degrade at
  /// runtime.
  pub fn load() -> Self
  {
    let engine = scene_script::build_engine();
    let dynamic : rhai::Dynamic = engine.eval( Self::SCRIPT )
    .expect( "scene.rhai is bundled at compile time and must evaluate" );
    let scene : Self = rhai::serde::from_dynamic( &dynamic )
    .expect( "scene.rhai's returned value must match SceneConfig's shape" );

    assert_eq!
    (
      scene.nebula_bands.len(), NEBULA_BAND_COUNT,
      "scene.rhai must declare exactly {NEBULA_BAND_COUNT} nebula_bands — scene.wgsl's uniform array is fixed-size"
    );
    assert_eq!
    (
      scene.star_layers.len(), STAR_LAYER_COUNT,
      "scene.rhai must declare exactly {STAR_LAYER_COUNT} star_layers — scene.wgsl's uniform array is fixed-size"
    );
    assert_eq!
    (
      scene.orbit_rings.len(), ORBIT_RING_COUNT,
      "scene.rhai must declare exactly {ORBIT_RING_COUNT} orbit_rings — scene.wgsl's uniform array is fixed-size"
    );
    assert_eq!
    (
      scene.nodes.len(), NODE_COUNT,
      "scene.rhai must declare exactly {NODE_COUNT} nodes — scene.wgsl's uniform array is fixed-size"
    );

    scene
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

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
}

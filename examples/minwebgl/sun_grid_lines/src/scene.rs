//! Deserializable scene configuration for the sun/grid HUD diagram, loaded
//! from `scene.rhai` at compile time via `include_str!` and evaluated
//! through `scene_script::build_engine()`. Kept free of any wasm/WebGL
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
#[ derive( Debug, Clone, Copy, Deserialize ) ]
pub struct Color( pub f64, pub f64, pub f64 );

impl Color
{
  /// Narrows to `[ r, g, b ]` — this crate uploads each color as its own
  /// `vec3` uniform (`gl::uniform::upload`, one call per field), unlike
  /// the WebGPU port's single packed `vec4`-padded uniform buffer, so no
  /// alpha/padding slot is needed here.
  pub fn to_array( self ) -> [ f32; 3 ]
  {
    [ self.0 as f32, self.1 as f32, self.2 as f32 ]
  }
}

#[ derive( Debug, Clone, Deserialize ) ]
pub struct Background
{
  pub top : Color,
  pub bottom : Color,
}

#[ derive( Debug, Clone, Deserialize ) ]
pub struct Nebula
{
  pub color : Color,
  pub opacity : f64,
}

#[ derive( Debug, Clone, Deserialize ) ]
pub struct Stars
{
  pub color : Color,
  pub intensity : f64,
}

#[ derive( Debug, Clone, Deserialize ) ]
pub struct Grid
{
  pub color : Color,
  pub opacity : f64,
}

#[ derive( Debug, Clone, Deserialize ) ]
pub struct SunCorona
{
  pub inner : Color,
  pub mid : Color,
  pub outer : Color,
}

#[ derive( Debug, Clone, Deserialize ) ]
pub struct SunDisc
{
  pub dark : Color,
  pub mid : Color,
  pub bright : Color,
  pub base_radius : f64,
}

#[ derive( Debug, Clone, Deserialize ) ]
pub struct OrbitRing
{
  pub color : Color,
  pub radius : f64,
}

/// The HUD diagram's static visual configuration — everything about it that
/// isn't already live-adjustable via the keyboard (seed, node count, grid
/// density stay runtime-interactive; see `Params` in `main.rs`).
#[ derive( Debug, Clone, Deserialize ) ]
pub struct SceneConfig
{
  pub background : Background,
  pub nebula : Nebula,
  pub stars : Stars,
  pub grid : Grid,
  pub sun_corona : SunCorona,
  pub sun_disc : SunDisc,
  pub orbit_ring : OrbitRing,
}

impl SceneConfig
{
  const SCRIPT : &'static str = include_str!( "../scene.rhai" );

  /// Evaluates the bundled `scene.rhai` and extracts a `SceneConfig` from
  /// its returned value via `rhai`'s serde bridge. Panics on a malformed
  /// script or a returned shape that doesn't match `SceneConfig` — the
  /// script is compiled into the binary by this crate itself, not supplied
  /// by an end user, so a failure here is a build-time authoring mistake
  /// that should fail loudly and immediately rather than degrade at
  /// runtime.
  pub fn load() -> Self
  {
    let engine = scene_script::build_engine();
    let dynamic : rhai::Dynamic = engine.eval( Self::SCRIPT )
    .expect( "scene.rhai is bundled at compile time and must evaluate" );
    rhai::serde::from_dynamic( &dynamic )
    .expect( "scene.rhai's returned value must match SceneConfig's shape" )
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  /// Asserts every field `scene.rhai` declares round-trips through
  /// `SceneConfig::load()` correctly — deliberately exhaustive ( not just a
  /// spot check ) so every field is exercised on every target, including
  /// native, where `main.rs`'s wasm32-gated `run()` — the only other
  /// consumer — never compiles in and can't do it instead.
  // Exact comparison is intentional: every value here is a literal parsed
  // straight out of scene.rhai with no arithmetic in between, so bit-exact
  // round-trip fidelity is exactly what this test means to check.
  #[ test ]
  fn scene_rhai_parses_and_matches_known_values()
  {
    let scene = SceneConfig::load();

    assert_eq!( scene.background.top.to_array(), [ 0.0196, 0.0549, 0.0941 ] );
    assert_eq!( scene.background.bottom.to_array(), [ 0.0549, 0.1490, 0.2392 ] );

    assert_eq!( scene.nebula.color.to_array(), [ 0.0706, 0.2000, 0.2902 ] );
    assert_eq!( scene.nebula.opacity, 0.45 );

    assert_eq!( scene.stars.color.to_array(), [ 0.6275, 0.8980, 1.0 ] );
    assert_eq!( scene.stars.intensity, 0.6 );

    assert_eq!( scene.grid.color.to_array(), [ 0.3137, 0.5490, 0.7451 ] );
    assert_eq!( scene.grid.opacity, 0.18 );

    assert_eq!( scene.sun_corona.inner.to_array(), [ 1.0, 0.8941, 0.4392 ] );
    assert_eq!( scene.sun_corona.mid.to_array(), [ 1.0, 0.6824, 0.1020 ] );
    assert_eq!( scene.sun_corona.outer.to_array(), [ 1.0, 0.2314, 0.0 ] );

    assert_eq!( scene.sun_disc.dark.to_array(), [ 1.0, 0.4157, 0.0 ] );
    assert_eq!( scene.sun_disc.mid.to_array(), [ 1.0, 0.8941, 0.4392 ] );
    assert_eq!( scene.sun_disc.bright.to_array(), [ 1.0, 1.0, 1.0 ] );
    assert_eq!( scene.sun_disc.base_radius, 0.075 );

    assert_eq!( scene.orbit_ring.color.to_array(), [ 0.3922, 0.8235, 1.0 ] );
    assert_eq!( scene.orbit_ring.radius, 0.425 );
  }
}

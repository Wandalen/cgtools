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
  #[ must_use ]
  pub fn to_array( self ) -> [ f32; 3 ]
  {
    [ self.0 as f32, self.1 as f32, self.2 as f32 ]
  }
}

/// Vertical background gradient endpoints.
#[ derive( Debug, Clone, Deserialize ) ]
pub struct Background
{
  /// Gradient color at the top edge.
  pub top : Color,
  /// Gradient color at the bottom edge.
  pub bottom : Color,
}

/// Nebula haze parameters.
#[ derive( Debug, Clone, Deserialize ) ]
pub struct Nebula
{
  /// Haze color.
  pub color : Color,
  /// Blend strength, 0..=1.
  pub opacity : f64,
}

/// Background star-field parameters.
#[ derive( Debug, Clone, Deserialize ) ]
pub struct Stars
{
  /// Star color.
  pub color : Color,
  /// Brightness multiplier.
  pub intensity : f64,
}

/// Polar HUD grid parameters.
#[ derive( Debug, Clone, Deserialize ) ]
pub struct Grid
{
  /// Line color.
  pub color : Color,
  /// Blend strength, 0..=1.
  pub opacity : f64,
}

/// Sun corona gradient stops.
#[ derive( Debug, Clone, Deserialize ) ]
pub struct SunCorona
{
  /// Color nearest the disc.
  pub inner : Color,
  /// Mid-falloff color.
  pub mid : Color,
  /// Outermost falloff color.
  pub outer : Color,
}

/// Sun disc shading and size.
#[ derive( Debug, Clone, Deserialize ) ]
pub struct SunDisc
{
  /// Limb ( edge ) color.
  pub dark : Color,
  /// Mid shading color.
  pub mid : Color,
  /// Core highlight color.
  pub bright : Color,
  /// Disc radius in normalized screen units.
  pub base_radius : f64,
}

/// Orbit ring parameters.
#[ derive( Debug, Clone, Deserialize ) ]
pub struct OrbitRing
{
  /// Ring color.
  pub color : Color,
  /// Ring radius in normalized screen units.
  pub radius : f64,
}

/// The HUD diagram's static visual configuration — everything about it that
/// isn't already live-adjustable via the keyboard (seed, node count, grid
/// density stay runtime-interactive; see `Params` in `main.rs`).
#[ derive( Debug, Clone, Deserialize ) ]
pub struct SceneConfig
{
  /// Vertical background gradient.
  pub background : Background,
  /// Nebula haze band.
  pub nebula : Nebula,
  /// Background star field.
  pub stars : Stars,
  /// Polar HUD grid.
  pub grid : Grid,
  /// Sun corona gradient.
  pub sun_corona : SunCorona,
  /// Sun disc shading.
  pub sun_disc : SunDisc,
  /// Orbit ring.
  pub orbit_ring : OrbitRing,
}

impl SceneConfig
{
  const SCRIPT : &'static str = include_str!( "../scene.rhai" );

  /// Evaluates the bundled `scene.rhai` and extracts a `SceneConfig` from
  /// its returned value via `rhai`'s serde bridge.
  ///
  /// # Panics
  ///
  /// Panics on a malformed script or a returned shape that doesn't match
  /// `SceneConfig` — the script is compiled into the binary by this crate
  /// itself, not supplied by an end user, so a failure here is a build-time
  /// authoring mistake that should fail loudly and immediately rather than
  /// degrade at runtime.
  #[ must_use ]
  pub fn load() -> Self
  {
    let engine = scene_script::build_engine();
    let dynamic : rhai::Dynamic = engine.eval( Self::SCRIPT )
    .expect( "scene.rhai is bundled at compile time and must evaluate" );
    rhai::serde::from_dynamic( &dynamic )
    .expect( "scene.rhai's returned value must match SceneConfig's shape" )
  }
}

//! Deserializable scene configuration for the sun/grid HUD diagram, loaded
//! from `scene.rhai` at compile time via `include_str!` and evaluated
//! through `scene_script::engine_build()`. Kept free of any wasm/WebGPU
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
  /// Narrows to `[ r, g, b, 1.0 ]` — the `vec4f`-packed layout
  /// `shader/scene_fragment.wgsl`'s `Uniforms` struct uses for every color
  /// field, sidestepping WGSL's vec3-aligns-to-16 padding rules entirely.
  #[ must_use ]
  pub fn to_array( self ) -> [ f32; 4 ]
  {
    [ self.0 as f32, self.1 as f32, self.2 as f32, 1.0 ]
  }
}

/// Vertical background gradient endpoints.
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct Background
{
  /// Gradient color at the top edge.
  pub top : Color,
  /// Gradient color at the bottom edge.
  pub bottom : Color,
}

/// One fog layer drifting across the vertical middle of the frame —
/// `scene.rhai` declares exactly [`NEBULA_BAND_COUNT`] of these.
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct NebulaBand
{
  /// Haze color.
  pub color : Color,
  /// Blend strength, 0..=1.
  pub opacity : f64,
  /// Vertical center of the band, 0..=1.
  pub center : f64,
  /// Vertical thickness of the band.
  pub thickness : f64,
  /// Noise frequency multiplier.
  pub noise_scale : f64,
  /// Horizontal drift speed ( sign gives direction ).
  pub drift_speed : f64,
}

/// One background star field: a hashed point per grid cell at `density`,
/// twinkling at `twinkle_speed` — `scene.rhai` declares exactly
/// [`STAR_LAYER_COUNT`] of these.
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct StarLayer
{
  /// Star color.
  pub color : Color,
  /// Brightness multiplier.
  pub intensity : f64,
  /// Grid-cell density of the star field.
  pub density : f64,
  /// Star point size.
  pub size : f64,
  /// Twinkle animation speed.
  pub twinkle_speed : f64,
}

/// Polar HUD grid parameters.
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct Grid
{
  /// Line color.
  pub color : Color,
  /// Blend strength, 0..=1.
  pub opacity : f64,
  /// Line width in pixels.
  pub line_width : f64,
  /// Glow strength around lines.
  pub glow : f64,
}

/// Sun corona gradient stops and flicker animation.
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct SunCorona
{
  /// Color nearest the disc.
  pub inner : Color,
  /// Mid-falloff color.
  pub mid : Color,
  /// Outermost falloff color.
  pub outer : Color,
  /// Radius where the inner color peaks, in normalized screen units.
  pub inner_radius : f64,
  /// Radius where the mid color peaks, in normalized screen units.
  pub mid_radius : f64,
  /// Radius where the corona fades out, in normalized screen units.
  pub outer_radius : f64,
  /// Flicker amplitude.
  pub flicker_amplitude : f64,
  /// Flicker animation speed.
  pub flicker_speed : f64,
}

/// Sun disc shading, size, and pulsation.
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
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
  /// Radius pulsation amplitude.
  pub pulsate_amplitude : f64,
  /// Radius pulsation speed.
  pub pulsate_speed : f64,
  /// Surface granulation noise frequency multiplier.
  pub granulation_scale : f64,
}

/// One concentric orbit rail — `scene.rhai` declares exactly
/// [`ORBIT_RING_COUNT`] of these.
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct OrbitRing
{
  /// Ring color.
  pub color : Color,
  /// Ring radius in normalized screen units.
  pub radius : f64,
  /// Glow strength around the ring stroke.
  pub glow : f64,
  /// Stroke width in pixels.
  pub stroke_width : f64,
  /// Pulse animation speed.
  pub pulse_speed : f64,
}

/// One authored planet/moon orbiting at a fixed `radius` with angular
/// `speed` (sign gives direction) and starting `phase` — independent of
/// the shader's own keyboard-driven procedural nodes (see
/// `shader/scene_fragment.wgsl`), which keep working unchanged.
/// `scene.rhai` declares exactly [`NODE_COUNT`] of these.
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct Node
{
  /// Node color.
  pub color : Color,
  /// Node size in normalized screen units.
  pub size : f64,
  /// Orbit radius in normalized screen units.
  pub radius : f64,
  /// Angular speed ( sign gives direction ).
  pub speed : f64,
  /// Starting angle in radians.
  pub phase : f64,
}

/// Cross-cutting single-pass effects applied after every primitive above is
/// composited. No offscreen texture exists to sample at an offset, so
/// effects are limited to what a single analytic pass can compute directly
/// (no true multi-tap effect like chromatic aberration).
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct Effects
{
  /// Vignette darkening strength.
  pub vignette_strength : f64,
  /// Radius where vignette darkening begins.
  pub vignette_radius : f64,
  /// Glow post-multiplier applied to bright areas.
  pub glow_intensity : f64,
  /// Scanline overlay strength.
  pub scanline_intensity : f64,
}

// List lengths `scene.rhai` must declare exactly, mirrored by
// `shader/scene_fragment.wgsl`'s `array<vec4f, N>` uniform fields — a WGSL
// `uniform` binding's arrays must be a compile-time fixed size, so there is
// no runtime element count to fall back on.

/// Fixed length of [`SceneConfig::nebula_bands`].
pub const NEBULA_BAND_COUNT : usize = 3;
/// Fixed length of [`SceneConfig::star_layers`].
pub const STAR_LAYER_COUNT : usize = 2;
/// Fixed length of [`SceneConfig::orbit_rings`].
pub const ORBIT_RING_COUNT : usize = 3;
/// Fixed length of [`SceneConfig::nodes`].
pub const NODE_COUNT : usize = 6;

/// The HUD diagram's full scene description — everything about it that
/// isn't already live-adjustable via the keyboard (seed, node count, grid
/// density stay runtime-interactive; see `Params` in `main.rs`).
#[ derive( Debug, Clone, PartialEq, Deserialize ) ]
pub struct SceneConfig
{
  /// Vertical background gradient.
  pub background : Background,
  /// Nebula haze bands ( exactly [`NEBULA_BAND_COUNT`] ).
  pub nebula_bands : Vec< NebulaBand >,
  /// Background star fields ( exactly [`STAR_LAYER_COUNT`] ).
  pub star_layers : Vec< StarLayer >,
  /// Polar HUD grid.
  pub grid : Grid,
  /// Sun corona gradient.
  pub sun_corona : SunCorona,
  /// Sun disc shading.
  pub sun_disc : SunDisc,
  /// Concentric orbit rails ( exactly [`ORBIT_RING_COUNT`] ).
  pub orbit_rings : Vec< OrbitRing >,
  /// Authored orbiting nodes ( exactly [`NODE_COUNT`] ).
  pub nodes : Vec< Node >,
  /// Cross-cutting post effects.
  pub effects : Effects,
}

impl SceneConfig
{
  const SCRIPT : &'static str = include_str!( "../scene/scene.rhai" );

  /// Evaluates the bundled `scene.rhai` and extracts a `SceneConfig` from
  /// its returned value via `rhai`'s serde bridge.
  ///
  /// # Panics
  ///
  /// Panics on a malformed script, a returned shape that doesn't match
  /// `SceneConfig`, or a list whose length doesn't match the fragment
  /// shader's fixed uniform array size — the script is compiled into the
  /// binary by this crate itself, not supplied by an end user, so a failure
  /// here is a build-time authoring mistake that should fail loudly and
  /// immediately rather than degrade at runtime.
  #[ must_use ]
  pub fn load() -> Self
  {
    let engine = scene_script::engine_build();
    let dynamic : rhai::Dynamic = engine.eval( Self::SCRIPT )
    .expect( "scene.rhai is bundled at compile time and must evaluate" );
    let scene : Self = rhai::serde::from_dynamic( &dynamic )
    .expect( "scene.rhai's returned value must match SceneConfig's shape" );

    assert_eq!
    (
      scene.nebula_bands.len(), NEBULA_BAND_COUNT,
      "scene.rhai must declare exactly {NEBULA_BAND_COUNT} nebula_bands — the fragment shader's uniform array is fixed-size"
    );
    assert_eq!
    (
      scene.star_layers.len(), STAR_LAYER_COUNT,
      "scene.rhai must declare exactly {STAR_LAYER_COUNT} star_layers — the fragment shader's uniform array is fixed-size"
    );
    assert_eq!
    (
      scene.orbit_rings.len(), ORBIT_RING_COUNT,
      "scene.rhai must declare exactly {ORBIT_RING_COUNT} orbit_rings — the fragment shader's uniform array is fixed-size"
    );
    assert_eq!
    (
      scene.nodes.len(), NODE_COUNT,
      "scene.rhai must declare exactly {NODE_COUNT} nodes — the fragment shader's uniform array is fixed-size"
    );

    scene
  }
}

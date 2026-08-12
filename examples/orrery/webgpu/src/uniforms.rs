//! GPU-side uniform buffer layout for the sun/grid HUD diagram, and its
//! construction from a loaded `scene::SceneConfig`. `time`/`seed`/
//! `node_count`/`grid_density`/`resolution` are the per-frame fields
//! refreshed every frame by `app_run()`'s animation loop via
//! `UniformsRaw::with_frame`; everything else is static scene styling loaded
//! once (see `impl From<&scene::SceneConfig>` below).

use orrery_webgpu::scene;
use minwebgpu as gl;

#[ repr( C ) ]
#[ derive( Clone, Copy, gl::mem::Pod, gl::mem::Zeroable ) ]
pub( crate ) struct UniformsRaw
{
  time : f32,
  seed : f32,
  node_count : i32,
  grid_density : f32,

  // Static scene styling below, loaded once from `scene.rhai` (see
  // `scene::SceneConfig`) and left unchanged every frame. Every field is
  // `[ f32; 4 ]` / `[ [ f32; 4 ]; N ]` to match the shader's ( `scene.rhai`'s
  // `shader` field ) `vec4f` / `array< vec4f, N >` fields — WGSL's uniform-buffer layout aligns
  // `vec3f` to 16 bytes and requires fixed-size arrays anyway, so packing
  // everything as vec4 slots avoids hand-deriving padding and keeps every
  // list's element count a compile-time constant on both sides.
  bg_top : [ f32; 4 ],
  bg_bottom : [ f32; 4 ],

  /// .xyz = color, .w = opacity
  nebula_colors : [ [ f32; 4 ]; scene::NEBULA_BAND_COUNT ],
  /// .x = vertical center, .y = thickness, .z = noise scale, .w = drift speed
  nebula_params : [ [ f32; 4 ]; scene::NEBULA_BAND_COUNT ],

  /// .xyz = color, .w = intensity
  star_colors : [ [ f32; 4 ]; scene::STAR_LAYER_COUNT ],
  /// .x = density, .y = point size, .z = twinkle speed, .w = unused
  star_params : [ [ f32; 4 ]; scene::STAR_LAYER_COUNT ],

  grid_color : [ f32; 4 ],
  /// x = opacity, y = line width, z = glow, w = unused
  grid_params : [ f32; 4 ],

  corona_inner : [ f32; 4 ],
  corona_mid : [ f32; 4 ],
  corona_outer : [ f32; 4 ],
  /// x = inner radius, y = mid radius, z = outer radius, w = unused
  corona_radii : [ f32; 4 ],
  /// x = flicker amplitude, y = flicker speed, zw = unused
  corona_flicker : [ f32; 4 ],

  disc_dark : [ f32; 4 ],
  disc_mid : [ f32; 4 ],
  disc_bright : [ f32; 4 ],
  /// x = base radius, y = pulsate amplitude, z = pulsate speed, w = granulation scale
  disc_params : [ f32; 4 ],

  /// .xyz = color, .w = glow amount
  ring_colors : [ [ f32; 4 ]; scene::ORBIT_RING_COUNT ],
  /// .x = radius, .y = stroke width, .z = pulse speed, .w = unused
  ring_params : [ [ f32; 4 ]; scene::ORBIT_RING_COUNT ],

  /// .xyz = color, .w = size
  node_colors : [ [ f32; 4 ]; scene::NODE_COUNT ],
  /// .x = orbit radius, .y = angular speed, .z = phase, .w = unused
  node_params : [ [ f32; 4 ]; scene::NODE_COUNT ],

  /// x = vignette strength, y = vignette radius, z = glow intensity, w = scanline intensity
  effects : [ f32; 4 ],

  /// .xy = drawing-buffer resolution in physical pixels, .zw = unused.
  /// Per-frame like `time` — the canvas fills its parent, so it changes
  /// whenever the page layout does — not scene styling.
  resolution : [ f32; 4 ],
}

impl UniformsRaw
{
  /// Returns a copy with the per-frame fields overwritten — `resolution` is
  /// the drawing-buffer size in physical pixels as a `( width, height )`
  /// tuple; every other ( static scene styling ) field carries over
  /// unchanged from `self`.
  pub( crate ) fn with_frame( &self, time : f32, seed : f32, node_count : i32, grid_density : f32, resolution : ( u32, u32 ) ) -> Self
  {
    let resolution = [ resolution.0 as f32, resolution.1 as f32, 0.0, 0.0 ];
    Self { time, seed, node_count, grid_density, resolution, ..*self }
  }
}

/// Packs a scene list — already asserted by `SceneConfig::load()` to have
/// exactly `N` elements, matching the shader's fixed-size uniform arrays —
/// into a `[ [ f32; 4 ]; N ]` uniform slot, one call per list, one closure
/// per `array<vec4f, N>` field.
fn packed< T, F, const N : usize >( items : &[ T ], pack : F ) -> [ [ f32; 4 ]; N ]
where
  F : Fn( &T ) -> [ f32; 4 ],
{
  core::array::from_fn( | i | pack( &items[ i ] ) )
}

impl From< &scene::SceneConfig > for UniformsRaw
{
  /// Builds the static-styling portion of `UniformsRaw` from a loaded scene —
  /// the per-frame fields ( `time`/`seed`/`node_count`/`grid_density`/
  /// `resolution` ) are left neutral, overwritten every frame via
  /// `UniformsRaw::with_frame`.
  fn from( scene : &scene::SceneConfig ) -> Self
  {
    Self
    {
      time : 0.0,
      seed : 0.0,
      node_count : 0,
      grid_density : 0.0,
      // 1x1 keeps the aspect term neutral before the first frame write.
      resolution : [ 1.0, 1.0, 0.0, 0.0 ],

      bg_top : scene.background.top.to_array(),
      bg_bottom : scene.background.bottom.to_array(),

      nebula_colors : packed( &scene.nebula_bands, | band | { let [ r, g, b, _ ] = band.color.to_array(); [ r, g, b, band.opacity as f32 ] } ),
      nebula_params : packed( &scene.nebula_bands, | band | [ band.center as f32, band.thickness as f32, band.noise_scale as f32, band.drift_speed as f32 ] ),

      star_colors : packed( &scene.star_layers, | layer | { let [ r, g, b, _ ] = layer.color.to_array(); [ r, g, b, layer.intensity as f32 ] } ),
      star_params : packed( &scene.star_layers, | layer | [ layer.density as f32, layer.size as f32, layer.twinkle_speed as f32, 0.0 ] ),

      grid_color : scene.grid.color.to_array(),
      grid_params : [ scene.grid.opacity as f32, scene.grid.line_width as f32, scene.grid.glow as f32, 0.0 ],

      corona_inner : scene.sun_corona.inner.to_array(),
      corona_mid : scene.sun_corona.mid.to_array(),
      corona_outer : scene.sun_corona.outer.to_array(),
      corona_radii : [ scene.sun_corona.inner_radius as f32, scene.sun_corona.mid_radius as f32, scene.sun_corona.outer_radius as f32, 0.0 ],
      corona_flicker : [ scene.sun_corona.flicker_amplitude as f32, scene.sun_corona.flicker_speed as f32, 0.0, 0.0 ],

      disc_dark : scene.sun_disc.dark.to_array(),
      disc_mid : scene.sun_disc.mid.to_array(),
      disc_bright : scene.sun_disc.bright.to_array(),
      disc_params : [ scene.sun_disc.base_radius as f32, scene.sun_disc.pulsate_amplitude as f32, scene.sun_disc.pulsate_speed as f32, scene.sun_disc.granulation_scale as f32 ],

      ring_colors : packed( &scene.orbit_rings, | ring | { let [ r, g, b, _ ] = ring.color.to_array(); [ r, g, b, ring.glow as f32 ] } ),
      ring_params : packed( &scene.orbit_rings, | ring | [ ring.radius as f32, ring.stroke_width as f32, ring.pulse_speed as f32, 0.0 ] ),

      node_colors : packed( &scene.nodes, | node | { let [ r, g, b, _ ] = node.color.to_array(); [ r, g, b, node.size as f32 ] } ),
      node_params : packed( &scene.nodes, | node | [ node.radius as f32, node.speed as f32, node.phase as f32, 0.0 ] ),

      effects : [ scene.effects.vignette_strength as f32, scene.effects.vignette_radius as f32, scene.effects.glow_intensity as f32, scene.effects.scanline_intensity as f32 ],
    }
  }
}

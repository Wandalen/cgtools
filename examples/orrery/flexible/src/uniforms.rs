//! Uniform buffer layout for the shared orrery scene shader, mirroring
//! `orrery_webgpu`'s own `uniforms.rs` field-for-field — both pack against
//! the same contract, `shader/scene_fragment.wgsl`'s `struct Uniforms`.
//! Reimplemented here ( not reused ) because gpu_hal's buffer API takes a
//! raw `&[u8]` ( `Queue::buffer_write` ), not `minwebgpu`'s
//! `Pod`/`Zeroable`-derived one, so the byte layout is produced by an
//! explicit `to_bytes` walk instead of a transmute-style derive.

use orrery_webgpu::scene;

/// Static-styling plus per-frame uniform fields, in `struct Uniforms`'
/// declared order — see [`UniformsRaw::to_bytes`] for the byte layout this
/// order produces.
// Fix(BUG-308): this struct's declared field order, `to_bytes()`'s push-call
// order below, and `orrery_webgpu`'s `shader/scene_fragment.wgsl` `Uniforms`
// struct order must all three stay in lockstep -- nothing else ties them
// together, since the buffer crosses to the GPU as raw bytes with no
// per-field validation anywhere. `tests/uniforms_layout_test.rs` guards all
// three; keep them synchronized when editing any one.
// Pitfall: `native_render_test.rs`/`vulkan_render_test.rs` only sample two
// pixels (sun-disc center, background corner) -- a same-shape field swap
// outside those regions (e.g. `ring_colors`/`ring_params`) compiles cleanly,
// preserves `to_bytes()`'s 704-byte `debug_assert_eq!`, and produces no
// render-test failure, only a silently corrupted buffer.
#[ repr( C ) ]
pub struct UniformsRaw
{
  time : f32,
  seed : f32,
  node_count : i32,
  grid_density : f32,

  bg_top : [ f32; 4 ],
  bg_bottom : [ f32; 4 ],

  nebula_colors : [ [ f32; 4 ]; scene::NEBULA_BAND_COUNT ],
  nebula_params : [ [ f32; 4 ]; scene::NEBULA_BAND_COUNT ],

  star_colors : [ [ f32; 4 ]; scene::STAR_LAYER_COUNT ],
  star_params : [ [ f32; 4 ]; scene::STAR_LAYER_COUNT ],

  grid_color : [ f32; 4 ],
  grid_params : [ f32; 4 ],

  corona_inner : [ f32; 4 ],
  corona_mid : [ f32; 4 ],
  corona_outer : [ f32; 4 ],
  corona_radii : [ f32; 4 ],
  corona_flicker : [ f32; 4 ],

  disc_dark : [ f32; 4 ],
  disc_mid : [ f32; 4 ],
  disc_bright : [ f32; 4 ],
  disc_params : [ f32; 4 ],

  ring_colors : [ [ f32; 4 ]; scene::ORBIT_RING_COUNT ],
  ring_params : [ [ f32; 4 ]; scene::ORBIT_RING_COUNT ],

  node_colors : [ [ f32; 4 ]; scene::NODE_COUNT ],
  node_params : [ [ f32; 4 ]; scene::NODE_COUNT ],

  effects : [ f32; 4 ],

  /// .xy = drawing-buffer resolution in physical pixels, .zw = unused.
  resolution : [ f32; 4 ],
}

/// Appends one `vec4f` slot, little-endian.
fn push_vec4( out : &mut Vec< u8 >, v : &[ f32; 4 ] )
{
  for x in v
  {
    out.extend_from_slice( &x.to_le_bytes() );
  }
}

/// Appends `N` consecutive `vec4f` slots, little-endian.
fn push_vec4_array< const N : usize >( out : &mut Vec< u8 >, arr : &[ [ f32; 4 ]; N ] )
{
  for v in arr
  {
    push_vec4( out, v );
  }
}

impl UniformsRaw
{
  /// Returns a copy with the per-frame fields overwritten — `resolution` is
  /// the drawing-buffer size in physical pixels as a `( width, height )`
  /// tuple; every other ( static scene styling ) field carries over
  /// unchanged from `self`.
  #[ must_use ]
  pub fn with_frame( &self, time : f32, seed : f32, node_count : i32, grid_density : f32, resolution : ( u32, u32 ) ) -> Self
  {
    let resolution = [ resolution.0 as f32, resolution.1 as f32, 0.0, 0.0 ];
    Self { time, seed, node_count, grid_density, resolution, ..*self }
  }

  /// Flattens every field, in `struct Uniforms`' declared order, into the
  /// 704-byte little-endian layout the shared shader expects — the
  /// `gpu_hal`-generic replacement for a `Pod`/`Zeroable` transmute.
  #[ must_use ]
  pub fn to_bytes( &self ) -> Vec< u8 >
  {
    let mut out = Vec::with_capacity( 704 );
    out.extend_from_slice( &self.time.to_le_bytes() );
    out.extend_from_slice( &self.seed.to_le_bytes() );
    out.extend_from_slice( &self.node_count.to_le_bytes() );
    out.extend_from_slice( &self.grid_density.to_le_bytes() );

    push_vec4( &mut out, &self.bg_top );
    push_vec4( &mut out, &self.bg_bottom );

    push_vec4_array( &mut out, &self.nebula_colors );
    push_vec4_array( &mut out, &self.nebula_params );

    push_vec4_array( &mut out, &self.star_colors );
    push_vec4_array( &mut out, &self.star_params );

    push_vec4( &mut out, &self.grid_color );
    push_vec4( &mut out, &self.grid_params );

    push_vec4( &mut out, &self.corona_inner );
    push_vec4( &mut out, &self.corona_mid );
    push_vec4( &mut out, &self.corona_outer );
    push_vec4( &mut out, &self.corona_radii );
    push_vec4( &mut out, &self.corona_flicker );

    push_vec4( &mut out, &self.disc_dark );
    push_vec4( &mut out, &self.disc_mid );
    push_vec4( &mut out, &self.disc_bright );
    push_vec4( &mut out, &self.disc_params );

    push_vec4_array( &mut out, &self.ring_colors );
    push_vec4_array( &mut out, &self.ring_params );

    push_vec4_array( &mut out, &self.node_colors );
    push_vec4_array( &mut out, &self.node_params );

    push_vec4( &mut out, &self.effects );
    push_vec4( &mut out, &self.resolution );

    debug_assert_eq!( out.len(), 704, "UniformsRaw::to_bytes must match struct Uniforms' 704-byte layout" );
    out
  }
}

impl From< &scene::SceneConfig > for UniformsRaw
{
  /// Builds the static-styling portion of `UniformsRaw` from a loaded scene —
  /// the per-frame fields ( `time`/`seed`/`node_count`/`grid_density`/
  /// `resolution` ) are left neutral, overwritten every frame via
  /// `UniformsRaw::with_frame`.
  fn from( scene : &scene::SceneConfig ) -> Self
  {
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

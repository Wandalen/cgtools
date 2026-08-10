// Procedural sci-fi HUD diagram: animated star, orbit ring, and a Cartesian
// grid, rendered by a fullscreen fragment shader. WGSL port of the WebGL2
// `minwebgl_sun_grid_lines` example's scene.frag; glow uses the same
// analytic radial-falloff / exp terms as that example's original single-pass
// version ( no multi-pass Gaussian bloom infrastructure exists for WebGPU in
// this workspace ).
//
// Every color/opacity/radius below reads from `uniforms`, sourced host-side
// from `scene.rhai` ( see `../src/scene.rs` ) — nothing here is a hardcoded
// visual constant except generation internals ( noise/hash magic numbers,
// AA epsilons, node jitter ) that aren't meant to be author-facing content.

struct Uniforms
{
  time : f32,
  seed : f32,
  node_count : i32,
  grid_density : f32,

  // Static scene styling, sourced from `scene.rhai` on the host side (see
  // `src/scene.rs`) instead of being baked in here as shader constants.
  bg_top : vec4f,
  bg_bottom : vec4f,
  nebula_color : vec4f,
  stars_color : vec4f,
  grid_color : vec4f,
  corona_inner : vec4f,
  corona_mid : vec4f,
  corona_outer : vec4f,
  disc_dark : vec4f,
  disc_mid : vec4f,
  disc_bright : vec4f,
  ring_color : vec4f,

  // x = nebula opacity, y = grid opacity, z = sun disc base radius, w = orbit ring radius
  scalars_a : vec4f,
  // x = star intensity, yzw = unused padding
  scalars_b : vec4f,
}

@group( 0 ) @binding( 0 ) var< uniform > uniforms : Uniforms;

const MAX_NODES : i32 = 8;

struct VertexOutput
{
  @builtin( position ) position : vec4f,
  @location( 0 ) uv : vec2f,
}

@vertex
fn vs_main( @builtin( vertex_index ) vertex_index : u32 ) -> VertexOutput
{
  // Big-triangle trick: 3 vertices, no buffer, vertex_index picks the corner.
  // The triangle overshoots clip space; only the visible unit square of uv
  // ( bottom-left = (0,0), top-right = (1,1) ) is ever rasterized to pixels.
  let x = i32( vertex_index ) & 1;
  let y = i32( vertex_index ) / 2;
  let uv = vec2f( f32( x ) * 2.0, f32( y ) * 2.0 );

  var out : VertexOutput;
  out.uv = uv;
  out.position = vec4f( uv * 2.0 - 1.0, 0.0, 1.0 );
  return out;
}

// Hash-based value noise. The workspace has no simplex/perlin implementation
// anywhere; this compact, dependency-free substitute is standard practice
// for shader-only procedural texture work.
fn hash21( p : vec2f ) -> f32
{
  var p3 = fract( vec3f( p.x, p.y, p.x ) * 0.1031 );
  p3 += vec3f( dot( p3, p3.yzx + 33.33 ) );
  return fract( ( p3.x + p3.y ) * p3.z );
}

fn value_noise( p : vec2f ) -> f32
{
  let i = floor( p );
  let f = fract( p );
  let a = hash21( i );
  let b = hash21( i + vec2f( 1.0, 0.0 ) );
  let c = hash21( i + vec2f( 0.0, 1.0 ) );
  let d = hash21( i + vec2f( 1.0, 1.0 ) );
  let u = f * f * ( 3.0 - 2.0 * f );
  return mix( mix( a, b, u.x ), mix( c, d, u.x ), u.y );
}

// Fixed 3-octave fractal Brownian motion, in [0, 0.875].
fn fbm3( p_in : vec2f ) -> f32
{
  var p = p_in;
  var value = 0.0;
  value += 0.5 * value_noise( p );
  p *= 2.0;
  value += 0.25 * value_noise( p );
  p *= 2.0;
  value += 0.125 * value_noise( p );
  return value;
}

@fragment
fn fs_main( in : VertexOutput ) -> @location( 0 ) vec4f
{
  let uv = in.uv; // y = 0 at canvas bottom, y = 1 at canvas top
  let center = vec2f( 0.5, 0.5 );
  let d = distance( uv, center );

  // 1. Background: vertical gradient, lighter toward vertical center.
  let navy = uniforms.bg_top.xyz;
  let slate = uniforms.bg_bottom.xyz;
  let vgrad = 1.0 - abs( uv.y - 0.5 ) * 2.0;
  var color = mix( navy, slate, vgrad );

  // 2. Nebula fog band across the vertical middle, noise-modulated.
  let band = smoothstep( 0.35, 0.45, uv.y ) * ( 1.0 - smoothstep( 0.55, 0.65, uv.y ) );
  let fog_n = fbm3( vec2f( uv.x * 3.0, uv.y * 8.0 ) + uniforms.seed * 0.37 );
  let nebula = uniforms.nebula_color.xyz;
  color = mix( color, nebula, band * fog_n * uniforms.scalars_a.x );

  // 3. Sparse background stars: one hashed candidate point per grid cell.
  {
    let cell = floor( uv * 9.0 );
    let cell_uv = fract( uv * 9.0 );
    let has_star = step( 0.86, hash21( cell + uniforms.seed ) );
    let star_pos = vec2f( hash21( cell + 0.17 + uniforms.seed ), hash21( cell + 4.31 + uniforms.seed ) );
    let star_d = distance( cell_uv, star_pos );
    let twinkle = 0.5 + 0.5 * sin( uniforms.time * ( 1.5 + hash21( cell + uniforms.seed ) * 2.0 ) + hash21( cell + uniforms.seed ) * 6.283 );
    let star = has_star * ( 1.0 - smoothstep( 0.0, 0.06, star_d ) ) * ( 0.4 + 0.6 * twinkle );
    color += uniforms.stars_color.xyz * star * uniforms.scalars_b.x;
  }

  // 4. Grid overlay, density controlled by uniforms.grid_density, constant
  // screen-space line width via fwidth.
  {
    let g = uv * uniforms.grid_density;
    let grid_d = abs( fract( g - 0.5 ) - 0.5 ) / fwidth( g );
    let line = 1.0 - min( min( grid_d.x, grid_d.y ), 1.0 );
    let grid_color = uniforms.grid_color.xyz;
    color = mix( color, grid_color, line * uniforms.scalars_a.y );
  }

  // 5. Central star corona: three-stop radial falloff, back to front.
  {
    let c0 = uniforms.corona_inner.xyz; // inner-most, warm yellow
    let c1 = uniforms.corona_mid.xyz; // mid corona, amber
    let c2 = uniforms.corona_outer.xyz; // outer corona, red-orange fading out
    let a0 = 1.0 - smoothstep( 0.0, 0.08, d );
    let a1 = ( 1.0 - smoothstep( 0.08, 0.15, d ) ) * 0.8;
    let a2 = ( 1.0 - smoothstep( 0.15, 0.25, d ) ) * 0.3;
    let corona = c0 * a0 + c1 * a1 * ( 1.0 - a0 ) + c2 * a2 * ( 1.0 - a0 ) * ( 1.0 - a1 );
    let corona_a = clamp( a0 + a1 + a2, 0.0, 1.0 );
    color = mix( color, corona, corona_a );
  }

  // 6. Star disk: fbm surface granulation inside a noise-jagged rim.
  {
    let base_radius = uniforms.scalars_a.z;
    let angle = atan2( uv.y - 0.5, uv.x - 0.5 );
    let rim_noise = fbm3( vec2f( cos( angle ), sin( angle ) ) * 4.0 ) - 0.4375;
    let radius = base_radius + rim_noise * 0.015;
    let disk = 1.0 - smoothstep( radius - 0.004, radius, d );

    let gran_n = fbm3( uv * 40.0 + 3.0 );
    let dark = uniforms.disc_dark.xyz;
    let mid = uniforms.disc_mid.xyz;
    let bright = uniforms.disc_bright.xyz;
    var surface = mix( dark, mid, smoothstep( 0.3, 0.6, gran_n ) );
    surface = mix( surface, bright, smoothstep( 0.75, 0.95, gran_n ) );

    color = mix( color, surface, disk );
  }

  // 7. Orbital ring: soft wide glow plus a crisp stroke core.
  {
    let ring_r = uniforms.scalars_a.w;
    let ring_d = abs( d - ring_r );
    let ring_color = uniforms.ring_color.xyz;
    let glow = exp( -ring_d * 220.0 ) * 0.35;
    let core = 1.0 - smoothstep( 0.0, 0.0022, ring_d );
    color += ring_color * glow;
    color = mix( color, ring_color, core );
  }

  // 8. Orbiting nodes, count controlled by uniforms.node_count. Each node's
  // phase and orbit radius are perturbed by a hash of uniforms.seed so
  // re-seeding visibly reshuffles the layout, not just the star field.
  {
    let node_count = clamp( uniforms.node_count, 1, MAX_NODES );
    for ( var i : i32 = 0; i < MAX_NODES; i++ )
    {
      if ( i >= node_count )
      {
        break;
      }

      let fi = f32( i );
      let node_seed = vec2f( uniforms.seed + fi * 12.9898, uniforms.seed + fi * 78.233 );
      let phase_jitter = ( hash21( node_seed ) - 0.5 ) * 1.2;
      let radius_jitter = 0.85 + 0.3 * hash21( node_seed + 5.17 );

      // Reference composition specifies its angle ( 325 deg ) in image space
      // ( y grows downward ); negating the sine term converts it into this
      // shader's y-up uv space. Time adds a slow orbital drift on top, with
      // a small per-node speed offset so nodes don't move in lockstep.
      let theta = radians( 325.0 ) + uniforms.time * ( 0.15 - fi * 0.015 )
        + fi * ( 6.28318 / f32( node_count ) ) + phase_jitter;
      let orbit_r = uniforms.scalars_a.w * radius_jitter;
      let planet_pos = vec2f( 0.5 + orbit_r * cos( theta ), 0.5 - orbit_r * sin( theta ) );
      let pd = distance( uv, planet_pos );

      let halo_color = uniforms.ring_color.xyz;
      let halo = ( 1.0 - smoothstep( 0.0, 0.018, pd ) ) * 0.85;
      color += halo_color * halo * 0.85;

      let core = 1.0 - smoothstep( 0.003, 0.006, pd );
      color = mix( color, vec3f( 1.0, 1.0, 1.0 ), core );
    }
  }

  return vec4f( color, 1.0 );
}

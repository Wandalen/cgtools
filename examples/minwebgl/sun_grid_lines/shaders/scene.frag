#version 300 es

precision highp float;

in vec2 v_uv;

uniform float u_time;
uniform float u_seed;
uniform int u_node_count;
uniform float u_grid_density;

// Static scene styling, sourced from `scene.rhai` on the host side ( see
// `../src/scene.rs` ) instead of being baked in here as shader constants.
// Nothing below is a hardcoded visual constant except generation internals
// ( noise/hash magic numbers, AA epsilons, node jitter ) that aren't meant
// to be author-facing content.
uniform vec3 u_bg_top;
uniform vec3 u_bg_bottom;
uniform vec3 u_nebula_color;
uniform float u_nebula_opacity;
uniform vec3 u_stars_color;
uniform float u_stars_intensity;
uniform vec3 u_grid_color;
uniform float u_grid_opacity;
uniform vec3 u_corona_inner;
uniform vec3 u_corona_mid;
uniform vec3 u_corona_outer;
uniform vec3 u_disc_dark;
uniform vec3 u_disc_mid;
uniform vec3 u_disc_bright;
uniform float u_disc_base_radius;
uniform vec3 u_ring_color;
uniform float u_ring_radius;

layout( location = 0 ) out vec4 frag_color;
// G-buffer-style emission output: only the layers that should bloom write
// here ( corona, star disk, ring, node halos ). Background/nebula/stars/grid
// are left black so the post-processing bloom pass never blurs them.
layout( location = 1 ) out vec4 frag_emission;

const int MAX_NODES = 8;

// Hash-based value noise. The workspace has no simplex/perlin implementation
// anywhere ( confirmed by search ); this compact, dependency-free substitute
// is standard practice for shader-only procedural texture work.
float hash21( vec2 p )
{
  vec3 p3 = fract( vec3( p.xyx ) * 0.1031 );
  p3 += dot( p3, p3.yzx + 33.33 );
  return fract( ( p3.x + p3.y ) * p3.z );
}

float value_noise( vec2 p )
{
  vec2 i = floor( p );
  vec2 f = fract( p );
  float a = hash21( i );
  float b = hash21( i + vec2( 1.0, 0.0 ) );
  float c = hash21( i + vec2( 0.0, 1.0 ) );
  float d = hash21( i + vec2( 1.0, 1.0 ) );
  vec2 u = f * f * ( 3.0 - 2.0 * f );
  return mix( mix( a, b, u.x ), mix( c, d, u.x ), u.y );
}

// Fixed 3-octave fractal Brownian motion, in [0, 0.875].
float fbm3( vec2 p )
{
  float value = 0.0;
  value += 0.5 * value_noise( p );
  p *= 2.0;
  value += 0.25 * value_noise( p );
  p *= 2.0;
  value += 0.125 * value_noise( p );
  return value;
}

void main()
{
  vec2 uv = v_uv; // y = 0 at canvas bottom, y = 1 at canvas top ( GL convention )
  vec2 center = vec2( 0.5 );
  float d = distance( uv, center );

  // 1. Background: vertical gradient, lighter toward vertical center.
  vec3 navy = u_bg_top;
  vec3 slate = u_bg_bottom;
  float vgrad = 1.0 - abs( uv.y - 0.5 ) * 2.0;
  vec3 color = mix( navy, slate, vgrad );
  vec3 emission = vec3( 0.0 );

  // 2. Nebula fog band across the vertical middle, noise-modulated.
  float band = smoothstep( 0.35, 0.45, uv.y ) * ( 1.0 - smoothstep( 0.55, 0.65, uv.y ) );
  float fog_n = fbm3( vec2( uv.x * 3.0, uv.y * 8.0 ) + u_seed * 0.37 );
  vec3 nebula = u_nebula_color;
  color = mix( color, nebula, band * fog_n * u_nebula_opacity );

  // 3. Sparse background stars: one hashed candidate point per grid cell.
  {
    vec2 cell = floor( uv * 9.0 );
    vec2 cell_uv = fract( uv * 9.0 );
    float has_star = step( 0.86, hash21( cell + u_seed ) );
    vec2 star_pos = vec2( hash21( cell + 0.17 + u_seed ), hash21( cell + 4.31 + u_seed ) );
    float star_d = distance( cell_uv, star_pos );
    float twinkle = 0.5 + 0.5 * sin( u_time * ( 1.5 + hash21( cell + u_seed ) * 2.0 ) + hash21( cell + u_seed ) * 6.283 );
    float star = has_star * ( 1.0 - smoothstep( 0.0, 0.06, star_d ) ) * ( 0.4 + 0.6 * twinkle );
    color += u_stars_color * star * u_stars_intensity;
  }

  // 4. Grid overlay, density controlled by u_grid_density, constant
  // screen-space line width via fwidth.
  {
    vec2 g = uv * u_grid_density;
    vec2 grid_d = abs( fract( g - 0.5 ) - 0.5 ) / fwidth( g );
    float line = 1.0 - min( min( grid_d.x, grid_d.y ), 1.0 );
    vec3 grid_color = u_grid_color;
    color = mix( color, grid_color, line * u_grid_opacity );
  }

  // 5. Central star corona: three-stop radial falloff, back to front.
  // Feeds emission at full strength — this is the scene's primary light source.
  {
    vec3 c0 = u_corona_inner; // inner-most, warm yellow
    vec3 c1 = u_corona_mid; // mid corona, amber
    vec3 c2 = u_corona_outer; // outer corona, red-orange fading out
    float a0 = 1.0 - smoothstep( 0.0, 0.08, d );
    float a1 = ( 1.0 - smoothstep( 0.08, 0.15, d ) ) * 0.8;
    float a2 = ( 1.0 - smoothstep( 0.15, 0.25, d ) ) * 0.3;
    vec3 corona = c0 * a0 + c1 * a1 * ( 1.0 - a0 ) + c2 * a2 * ( 1.0 - a0 ) * ( 1.0 - a1 );
    float corona_a = clamp( a0 + a1 + a2, 0.0, 1.0 );
    color = mix( color, corona, corona_a );
    emission = mix( emission, corona, corona_a );
  }

  // 6. Star disk: fbm surface granulation inside a noise-jagged rim.
  {
    float base_radius = u_disc_base_radius;
    float angle = atan( uv.y - 0.5, uv.x - 0.5 );
    float rim_noise = fbm3( vec2( cos( angle ), sin( angle ) ) * 4.0 ) - 0.4375;
    float radius = base_radius + rim_noise * 0.015;
    float disk = 1.0 - smoothstep( radius - 0.004, radius, d );

    float gran_n = fbm3( uv * 40.0 + 3.0 );
    vec3 dark = u_disc_dark;
    vec3 mid = u_disc_mid;
    vec3 bright = u_disc_bright;
    vec3 surface = mix( dark, mid, smoothstep( 0.3, 0.6, gran_n ) );
    surface = mix( surface, bright, smoothstep( 0.75, 0.95, gran_n ) );

    color = mix( color, surface, disk );
    emission = mix( emission, surface, disk );
  }

  // 7. Orbital ring: soft wide glow plus a crisp stroke core.
  {
    float ring_r = u_ring_radius;
    float ring_d = abs( d - ring_r );
    vec3 ring_color = u_ring_color;
    float glow = exp( -ring_d * 220.0 ) * 0.35;
    float core = 1.0 - smoothstep( 0.0, 0.0022, ring_d );
    color += ring_color * glow;
    color = mix( color, ring_color, core );
    emission += ring_color * glow;
    emission = mix( emission, ring_color, core );
  }

  // 8. Orbiting nodes, count controlled by u_node_count. Each node's phase
  // and orbit radius are perturbed by a hash of u_seed so re-seeding
  // ( see readme ) visibly reshuffles the layout, not just the star field.
  {
    int node_count = clamp( u_node_count, 1, MAX_NODES );
    for ( int i = 0; i < MAX_NODES; i++ )
    {
      if ( i >= node_count )
      {
        break;
      }

      float fi = float( i );
      vec2 node_seed = vec2( u_seed + fi * 12.9898, u_seed + fi * 78.233 );
      float phase_jitter = ( hash21( node_seed ) - 0.5 ) * 1.2;
      float radius_jitter = 0.85 + 0.3 * hash21( node_seed + 5.17 );

      // Reference composition specifies its angle ( 325 deg ) in image space
      // ( y grows downward ); negating the sine term converts it into this
      // shader's y-up uv space. u_time adds a slow orbital drift on top,
      // with a small per-node speed offset so nodes don't move in lockstep.
      float theta = radians( 325.0 ) + u_time * ( 0.15 - fi * 0.015 )
        + fi * ( 6.28318 / float( node_count ) ) + phase_jitter;
      float orbit_r = u_ring_radius * radius_jitter;
      vec2 planet_pos = vec2( 0.5 + orbit_r * cos( theta ), 0.5 - orbit_r * sin( theta ) );
      float pd = distance( uv, planet_pos );

      vec3 halo_color = u_ring_color;
      float halo = ( 1.0 - smoothstep( 0.0, 0.018, pd ) ) * 0.85;
      color += halo_color * halo * 0.85;
      emission += halo_color * halo * 0.85;

      float core = 1.0 - smoothstep( 0.003, 0.006, pd );
      color = mix( color, vec3( 1.0 ), core );
      emission = mix( emission, vec3( 1.0 ), core );
    }
  }

  frag_color = vec4( color, 1.0 );
  frag_emission = vec4( emission, 1.0 );
}

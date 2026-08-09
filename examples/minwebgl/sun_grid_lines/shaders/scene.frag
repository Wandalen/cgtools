#version 300 es

precision highp float;

in vec2 v_uv;

uniform float u_time;

out vec4 frag_color;

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
  vec3 navy = vec3( 0.0196, 0.0549, 0.0941 );
  vec3 slate = vec3( 0.0549, 0.1490, 0.2392 );
  float vgrad = 1.0 - abs( uv.y - 0.5 ) * 2.0;
  vec3 color = mix( navy, slate, vgrad );

  // 2. Nebula fog band across the vertical middle, noise-modulated.
  float band = smoothstep( 0.35, 0.45, uv.y ) * ( 1.0 - smoothstep( 0.55, 0.65, uv.y ) );
  float fog_n = fbm3( vec2( uv.x * 3.0, uv.y * 8.0 ) );
  vec3 nebula = vec3( 0.0706, 0.2000, 0.2902 );
  color = mix( color, nebula, band * fog_n * 0.45 );

  // 3. Sparse background stars: one hashed candidate point per grid cell.
  {
    vec2 cell = floor( uv * 9.0 );
    vec2 cell_uv = fract( uv * 9.0 );
    float has_star = step( 0.86, hash21( cell ) );
    vec2 star_pos = vec2( hash21( cell + 0.17 ), hash21( cell + 4.31 ) );
    float star_d = distance( cell_uv, star_pos );
    float twinkle = 0.5 + 0.5 * sin( u_time * ( 1.5 + hash21( cell ) * 2.0 ) + hash21( cell ) * 6.283 );
    float star = has_star * ( 1.0 - smoothstep( 0.0, 0.06, star_d ) ) * ( 0.4 + 0.6 * twinkle );
    color += vec3( 0.6275, 0.8980, 1.0000 ) * star * 0.6;
  }

  // 4. 10x10 grid overlay, constant screen-space line width via fwidth.
  {
    vec2 g = uv * 10.0;
    vec2 grid_d = abs( fract( g - 0.5 ) - 0.5 ) / fwidth( g );
    float line = 1.0 - min( min( grid_d.x, grid_d.y ), 1.0 );
    vec3 grid_color = vec3( 0.3137, 0.5490, 0.7451 );
    color = mix( color, grid_color, line * 0.18 );
  }

  // 5. Central star corona: three-stop radial falloff, back to front.
  {
    vec3 c0 = vec3( 1.0000, 0.8941, 0.4392 ); // inner-most, warm yellow
    vec3 c1 = vec3( 1.0000, 0.6824, 0.1020 ); // mid corona, amber
    vec3 c2 = vec3( 1.0000, 0.2314, 0.0000 ); // outer corona, red-orange fading out
    float a0 = 1.0 - smoothstep( 0.0, 0.08, d );
    float a1 = ( 1.0 - smoothstep( 0.08, 0.15, d ) ) * 0.8;
    float a2 = ( 1.0 - smoothstep( 0.15, 0.25, d ) ) * 0.3;
    vec3 corona = c0 * a0 + c1 * a1 * ( 1.0 - a0 ) + c2 * a2 * ( 1.0 - a0 ) * ( 1.0 - a1 );
    float corona_a = clamp( a0 + a1 + a2, 0.0, 1.0 );
    color = mix( color, corona, corona_a );
  }

  // 6. Star disk: fbm surface granulation inside a noise-jagged rim.
  {
    float base_radius = 0.075;
    float angle = atan( uv.y - 0.5, uv.x - 0.5 );
    float rim_noise = fbm3( vec2( cos( angle ), sin( angle ) ) * 4.0 ) - 0.4375;
    float radius = base_radius + rim_noise * 0.015;
    float disk = 1.0 - smoothstep( radius - 0.004, radius, d );

    float gran_n = fbm3( uv * 40.0 + 3.0 );
    vec3 dark = vec3( 1.0000, 0.4157, 0.0000 );
    vec3 mid = vec3( 1.0000, 0.8941, 0.4392 );
    vec3 bright = vec3( 1.0, 1.0, 1.0 );
    vec3 surface = mix( dark, mid, smoothstep( 0.3, 0.6, gran_n ) );
    surface = mix( surface, bright, smoothstep( 0.75, 0.95, gran_n ) );

    color = mix( color, surface, disk );
  }

  // 7. Orbital ring: soft wide glow plus a crisp stroke core.
  {
    float ring_r = 0.425;
    float ring_d = abs( d - ring_r );
    vec3 ring_color = vec3( 0.3922, 0.8235, 1.0000 );
    float glow = exp( -ring_d * 220.0 ) * 0.35;
    float core = 1.0 - smoothstep( 0.0, 0.0022, ring_d );
    color += ring_color * glow;
    color = mix( color, ring_color, core );
  }

  // 8. Orbiting planet node, slowly circling the ring.
  {
    // Reference composition specifies its angle ( 325 deg ) in image space
    // ( y grows downward ); negating the sine term converts it into this
    // shader's y-up uv space. u_time adds a slow orbital drift on top.
    float theta = radians( 325.0 ) + u_time * 0.15;
    float orbit_r = 0.425;
    vec2 planet_pos = vec2( 0.5 + orbit_r * cos( theta ), 0.5 - orbit_r * sin( theta ) );
    float pd = distance( uv, planet_pos );

    vec3 halo_color = vec3( 0.3922, 0.8235, 1.0000 );
    float halo = ( 1.0 - smoothstep( 0.0, 0.018, pd ) ) * 0.85;
    color += halo_color * halo * 0.85;

    float core = 1.0 - smoothstep( 0.003, 0.006, pd );
    color = mix( color, vec3( 1.0 ), core );
  }

  frag_color = vec4( color, 1.0 );
}

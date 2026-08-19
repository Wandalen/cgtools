#version 300 es
precision highp float;

// Nebula sky-dome backdrop, standing in for `scene/world.js`'s flat
// `scene.background = new THREE.Color(COLORS.spaceBg)` - the reference game
// screenshot the tactical UI is modeled on has soft drifting cloud blobs
// over that same base blue, not a flat fill.

in vec2 v_ndc;

uniform mat4 u_inv_view_proj;
uniform vec3 u_camera_position;
uniform float u_time;

out vec4 frag_color;

// 3D (not 2D angle-based) hash/noise - sampled directly off the ray
// direction below. An azimuth/elevation angle mapping has a pole
// singularity straight up/down (atan2 spins arbitrarily fast as x and z
// both approach 0), which shows up as visible seam lines radiating from the
// top/bottom of the sky when looking straight up or down. Noise in 3D has
// no such pole, so it stays seamless over the whole direction sphere.
float hash3( vec3 p )
{
  p = fract( p * vec3( 443.897, 441.423, 437.195 ) );
  p += dot( p, p.yzx + 19.19 );
  return fract( ( p.x + p.y ) * p.z );
}

float value_noise3( vec3 p )
{
  vec3 i = floor( p );
  vec3 f = fract( p );
  vec3 u = f * f * ( 3.0 - 2.0 * f );

  float a = hash3( i + vec3( 0.0, 0.0, 0.0 ) );
  float b = hash3( i + vec3( 1.0, 0.0, 0.0 ) );
  float c = hash3( i + vec3( 0.0, 1.0, 0.0 ) );
  float d = hash3( i + vec3( 1.0, 1.0, 0.0 ) );
  float e = hash3( i + vec3( 0.0, 0.0, 1.0 ) );
  float f2 = hash3( i + vec3( 1.0, 0.0, 1.0 ) );
  float g = hash3( i + vec3( 0.0, 1.0, 1.0 ) );
  float h = hash3( i + vec3( 1.0, 1.0, 1.0 ) );

  float bottom = mix( mix( a, b, u.x ), mix( c, d, u.x ), u.y );
  float top = mix( mix( e, f2, u.x ), mix( g, h, u.x ), u.y );
  return mix( bottom, top, u.z );
}

float fbm( vec3 p )
{
  float sum = 0.0;
  float amp = 0.5;
  for ( int i = 0; i < 5; i++ )
  {
    sum += amp * value_noise3( p );
    p *= 2.02;
    amp *= 0.5;
  }
  return sum;
}

void main()
{
  // Reconstruct the world-space ray through this pixel (far plane, then back
  // to the camera) and use its direction rather than distance/UV, so the
  // clouds read as a fixed backdrop attached to world space instead of
  // screen-space decals that swim as the orbit camera moves.
  vec4 far = u_inv_view_proj * vec4( v_ndc, 1.0, 1.0 );
  far.xyz /= far.w;
  vec3 ray_dir = normalize( far.xyz - u_camera_position );

  float elevation = ray_dir.y;
  vec3 sky_pos = ray_dir * 2.5 + vec3( u_time * 0.02, 0.0, u_time * 0.008 );

  float clouds = fbm( sky_pos );
  clouds = smoothstep( 0.35, 0.85, clouds );

  // Lightened past COLORS.spaceBg (0x14405c, the JS reference's flat fill)
  // toward the reference screenshot's brighter light-blue field - `base` is
  // the tint that should read as the sky's main color almost everywhere;
  // `deep` only shows up as a slight darkening right at the bottom of the
  // frame, not the strong top/bottom split a wide elevation gradient gave.
  vec3 deep = vec3( 0.08, 0.20, 0.30 );
  vec3 base = vec3( 0.18, 0.38, 0.52 );
  vec3 bright = vec3( 0.55, 0.74, 0.85 );

  vec3 color = mix( deep, base, smoothstep( -0.9, -0.15, elevation ) );
  color = mix( color, bright, clouds * 0.5 );

  // Soft falloff right at the bottom edge only, echoing the reference
  // screenshot's letterbox vignette without darkening the rest of the sky.
  float vignette = 1.0 - 0.25 * smoothstep( 0.55, 1.0, -v_ndc.y );
  color *= vignette;

  frag_color = vec4( color, 1.0 );
}

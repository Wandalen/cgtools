#version 300 es
precision highp float;

// Flat-shaded via screen-space derivatives of world position (same trick
// three.js's `flatShading: true` uses internally) instead of duplicating
// vertices per face for hard face normals. Shared by every hard-surface
// part in the scene (asteroids, ship hulls, station modules) - u_ambient
// is what tells them apart: a low ambient (asteroids/hull/dark parts) reads
// as normally lit geometry, while ambient = 1.0 (engine glow, beacon light)
// makes a part read as fully self-lit regardless of its facing or shadowing,
// standing in for three.js's separate unlit `MeshBasicMaterial` glow parts.
in vec3 v_world_pos;

uniform vec3 u_color;
uniform float u_ambient;
uniform vec3 u_camera_position;
uniform vec3 u_light_dir;
uniform vec3 u_light_color;
uniform float u_light_intensity;
uniform mat4 u_light_view_proj;
uniform sampler2D u_shadow_map;
uniform float u_shadows_enabled;

out vec4 frag_color;

// Cool tint blended into the unlit portion of a surface - the reference
// screenshot's white hull plating reads with a strong warm-lit highlight
// and a distinctly blueish shadow side, not a uniformly-darkened one.
const vec3 AMBIENT_TINT = vec3( 0.55, 0.68, 0.95 );
const float SHININESS = 48.0;
const float SPECULAR_STRENGTH = 0.6;

// 3x3 PCF around the fragment's shadow-map texel - a plain single-tap
// comparison reads as visibly jagged/aliased at this shadow-map resolution,
// especially on the low-poly hull silhouettes. Returns 1.0 = fully lit,
// 0.0 = fully shadowed; texels outside the light's frustum count as lit
// rather than shadowed, so geometry outside the shadow-casting region
// (which is deliberately sized to just the ships/station/asteroid cluster,
// not the whole grid) doesn't fall back to reading as permanently shadowed.
//
// Deliberately no depth bias: `ShadowMap::bind()` (renderer's shadow.rs)
// already renders the depth pass with front-face culling, so the map holds
// each caster's *back*-face depth - a receiver's own front face is always
// nearer the light than that by construction, which is the standard
// front-face-culling trick for avoiding self-shadow acne without a bias
// term at all. Stacking a slope-scaled bias on top of that (an earlier
// version of this function did) double-compensates: at grazing angles the
// bias grows large enough to overshoot past the already-correct back-face
// offset, and because it varies smoothly across a flat face while the PCF
// taps blend in correctly-unlit neighbor texels near the silhouette, the
// result was a strange "lit edges, dark blotchy center" pattern on a single
// flat face instead of a clean shadow.
float shadow_factor( vec3 world_pos )
{
  vec4 light_clip = u_light_view_proj * vec4( world_pos, 1.0 );
  vec3 proj = light_clip.xyz / light_clip.w;
  proj = proj * 0.5 + 0.5;

  if ( proj.x < 0.0 || proj.x > 1.0 || proj.y < 0.0 || proj.y > 1.0 || proj.z > 1.0 )
  {
    return 1.0;
  }

  vec2 texel = 1.0 / vec2( textureSize( u_shadow_map, 0 ) );

  float lit = 0.0;
  for ( int x = -1; x <= 1; x++ )
  {
    for ( int y = -1; y <= 1; y++ )
    {
      float depth = texture( u_shadow_map, proj.xy + vec2( x, y ) * texel ).r;
      lit += ( proj.z > depth ) ? 0.0 : 1.0;
    }
  }
  return lit / 9.0;
}

void main()
{
  vec3 normal = normalize( cross( dFdx( v_world_pos ), dFdy( v_world_pos ) ) );
  if ( !gl_FrontFacing ) normal = -normal;

  vec3 light_dir = normalize( u_light_dir );
  float n_dot_l = max( dot( normal, light_dir ), 0.0 );
  float shadow = mix( 1.0, shadow_factor( v_world_pos ), u_shadows_enabled );
  float lit = n_dot_l * shadow;

  vec3 view_dir = normalize( u_camera_position - v_world_pos );
  vec3 half_dir = normalize( light_dir + view_dir );
  float spec = pow( max( dot( normal, half_dir ), 0.0 ), SHININESS ) * lit;

  // u_ambient floors the shaded result (asteroids/hull plating: 0.35) so
  // shadowed geometry is dim, blue-tinted, but never pure black.
  float floor_and_diffuse = u_ambient + ( 1.0 - u_ambient ) * lit;
  vec3 ambient_color = u_color * AMBIENT_TINT;
  vec3 diffuse_color = u_color * u_light_color * u_light_intensity;
  vec3 base = mix( ambient_color, diffuse_color, lit ) * floor_and_diffuse;

  // Fully self-lit parts (u_ambient == 1.0: engine glow, beacon) fall
  // straight through to plain u_color regardless of `base` above, same
  // invariant the original flat-ambient formula relied on - `self_lit` is a
  // hard step rather than a continuous blend since AMBIENT_LIT/AMBIENT_GLOW
  // (see hull.rs) are the only two values ever passed in, and blending
  // partway between `base` and `u_color` for the *normal* 0.35 case would
  // just dilute the shading effect this rewrite is for.
  float self_lit = step( 0.999, u_ambient );
  vec3 color = mix( base, u_color, self_lit );
  color += u_light_color * u_light_intensity * spec * SPECULAR_STRENGTH * ( 1.0 - self_lit );

  frag_color = vec4( color, 1.0 );
}

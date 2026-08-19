#version 300 es
precision highp float;

// Flat-shaded via screen-space derivatives of world position (same trick
// three.js's `flatShading: true` uses internally) instead of duplicating
// vertices per face for hard face normals. Shared by every hard-surface
// part in the scene (asteroids, ship hulls, station modules) - u_ambient
// is what tells them apart: a low ambient (asteroids/hull/dark parts) reads
// as normally lit geometry, while ambient = 1.0 (engine glow, beacon light)
// makes a part read as fully self-lit regardless of its facing, standing in
// for three.js's separate unlit `MeshBasicMaterial` glow parts.
in vec3 v_world_pos;

uniform vec3 u_color;
uniform vec3 u_light_dir;
uniform float u_ambient;

out vec4 frag_color;

void main()
{
  vec3 normal = normalize( cross( dFdx( v_world_pos ), dFdy( v_world_pos ) ) );
  if ( !gl_FrontFacing ) normal = -normal;

  float n_dot_l = max( dot( normal, normalize( u_light_dir ) ), 0.0 );
  vec3 color = u_color * ( u_ambient + ( 1.0 - u_ambient ) * n_dot_l );

  frag_color = vec4( color, 1.0 );
}

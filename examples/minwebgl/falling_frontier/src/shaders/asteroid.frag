#version 300 es
precision highp float;

// Flat-shaded via screen-space derivatives of world position (same trick
// three.js's `flatShading: true` uses internally) instead of duplicating
// vertices per face for hard face normals - matches the low-poly deformed
// "rock" look of the JS reference's DodecahedronGeometry + deformToRock().
in vec3 v_world_pos;

uniform vec3 u_color;
uniform vec3 u_light_dir;

out vec4 frag_color;

void main()
{
  vec3 normal = normalize( cross( dFdx( v_world_pos ), dFdy( v_world_pos ) ) );
  if ( !gl_FrontFacing ) normal = -normal;

  float n_dot_l = max( dot( normal, normalize( u_light_dir ) ), 0.0 );
  float ambient = 0.35;
  vec3 color = u_color * ( ambient + ( 1.0 - ambient ) * n_dot_l );

  frag_color = vec4( color, 1.0 );
}

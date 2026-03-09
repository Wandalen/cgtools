#version 300 es
precision highp float;

// Per-vertex (from geometry VAO)
layout( location = 0 ) in vec2 a_position;
layout( location = 1 ) in vec2 a_uv;

// Per-instance (divisor = 1)
layout( location = 2 ) in vec3 i_transform_0;
layout( location = 3 ) in vec3 i_transform_1;
layout( location = 4 ) in vec3 i_transform_2;

uniform vec2 u_viewport;
uniform mat3 u_parent; // batch parent transform

out vec2 v_uv;
out vec2 v_pos;

void main()
{
  v_uv = a_uv;

  mat3 inst = transpose( mat3( i_transform_0, i_transform_1, i_transform_2 ) );
  vec3 world = u_parent * inst * vec3( a_position, 1.0 );

  vec2 ndc = ( world.xy / u_viewport ) * 2.0 - 1.0;
  ndc.y = -ndc.y;

  v_pos = world.xy;
  gl_Position = vec4( ndc, 0.0, 1.0 );
}

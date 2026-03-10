#version 300 es
precision highp float;

layout( location = 0 ) in vec2 a_position;
layout( location = 1 ) in vec2 a_uv; // optional, zero if no UVs

uniform mat3 u_transform;
uniform vec2 u_viewport;

out vec2 v_uv;
out vec2 v_pos;

void main()
{
  v_uv = a_uv;

  vec3 world = u_transform * vec3( a_position, 1.0 );

  vec2 ndc = ( world.xy / u_viewport ) * 2.0 - 1.0;

  v_pos = world.xy;
  gl_Position = vec4( ndc, 0.0, 1.0 );
}

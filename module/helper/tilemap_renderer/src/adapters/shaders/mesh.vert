#version 300 es
precision highp float;

layout( location = 0 ) in vec2 a_position;
layout( location = 1 ) in vec2 a_uv; // optional, zero if no UVs

uniform mat3 u_transform;
uniform vec2 u_viewport;
uniform float u_depth;       // NDC z; higher Transform::depth → drawn on top

out vec2 v_uv;

void main()
{
  v_uv = a_uv;

  vec3 world = u_transform * vec3( a_position, 1.0 );

  vec2 ndc = ( world.xy / u_viewport ) * 2.0 - 1.0;

  // Negate so higher user depth → smaller clip-space z → wins LEQUAL depth test.
  gl_Position = vec4( ndc, -u_depth, 1.0 );
}

#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec2 inPoint;

uniform mat3 u_world_matrix;
uniform mat3 u_view_matrix;
uniform mat4 u_projection_matrix;
uniform float u_width;

void main() 
{
  vec2 point = ( u_world_matrix * vec3( inPoint, 1.0 ) ).xy;
  point += u_width * position;

  vec3 view_point = u_view_matrix * vec3( point, 1.0 );
  gl_Position = u_projection_matrix * vec4( view_point.xy, 0.0, 1.0 );
}
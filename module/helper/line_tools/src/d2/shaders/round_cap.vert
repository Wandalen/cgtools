#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;



uniform vec2 u_inPoint;
uniform mat3 u_world_matrix;
uniform mat4 u_projection_matrix;
uniform float u_width;

void main() 
{
  vec2 u_point = ( u_world_matrix * vec3( u_inPoint, 1.0 ) ).xy;

  gl_Position =  u_projection_matrix * vec4( u_point + u_width * position, 0.0, 1.0 );
}
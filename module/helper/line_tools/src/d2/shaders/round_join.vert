#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec2 point;

uniform mat4 u_projection_matrix;
uniform float u_width;

void main() 
{
  gl_Position =  u_projection_matrix * vec4( position * u_width + point, 0.0, 1.0 );
}
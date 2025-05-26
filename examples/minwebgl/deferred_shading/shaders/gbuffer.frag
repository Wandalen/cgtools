#version 300 es

precision mediump float;

in vec3 v_position;
in vec3 v_normal;

layout( location = 0 ) out vec4 position;
layout( location = 1 ) out vec4 normal;

void main()
{
  position = vec4( v_position, 1.0 );
  normal = vec4( normalize( v_normal ), 1.0 );
}

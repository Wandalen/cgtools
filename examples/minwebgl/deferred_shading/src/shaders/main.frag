#version 300 es
precision mediump float;

in vec3 v_normal;
in vec3 v_world_pos;

layout( location = 0 ) out vec3 position;
layout( location = 1 ) out vec3 normal;
layout( location = 2 ) out vec4 color;

void main()
{
  position = v_world_pos;
  normal = normalize( v_normal );
  color = vec4( 0.0, 1.0, 0.0, 1.0 );
}

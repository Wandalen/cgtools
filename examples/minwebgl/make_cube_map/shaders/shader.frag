#version 300 es

precision mediump float;

uniform samplerCube cube_map;
uniform float max_distance;

in vec3 vNormal;
in vec3 vPosition;

out vec4 frag_color;

void main()
{
  frag_color = texture( cube_map, vPosition );
}


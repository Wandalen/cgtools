#version 300 es
precision mediump float;

uniform int u_id;

layout( location = 0 ) out int instance_id;

void main()
{
  instance_id = u_id;
}

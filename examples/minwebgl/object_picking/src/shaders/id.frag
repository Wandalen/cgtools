#version 300 es
precision mediump float;

uniform int u_id;

layout( location = 0 ) out int instance_id;

void main()
{
  // this shader is meant for drawing object's id into texture
  // this just wrties object's id into texture
  instance_id = u_id;
}

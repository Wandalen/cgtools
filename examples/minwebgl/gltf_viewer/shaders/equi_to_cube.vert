#version 300 es

layout( 0 ) vec3 pos;

uniform mat4x4 projectionMatrix;
uniform mat4x4 viewMatrix;

out vec3 localPos;

void main()
{
  localPos = pos;
  gl_Position = projectionMatrix * viewMatrix * vec4( pos, 1.0 );
}
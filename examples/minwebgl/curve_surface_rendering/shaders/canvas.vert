#version 300 es

layout( location = 0 ) in vec3 position;

uniform mat4x4 worldMatrix;
uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;

void main()
{
  vec4 worldPos = worldMatrix * vec4( position, 1.0 );
  worldPos.y = - worldPos.y;
  vec4 viewPos = viewMatrix * worldPos;
  gl_Position = projectionMatrix * viewPos;
}

#version 300 es

// Slot 0 is `positions` for every primitive the glTF loader builds, so one program
// covers every mesh in a scene regardless of which other attributes it carries.
layout ( location = 0 ) in vec3 position;

uniform mat4 viewProjectionMatrix;
uniform mat4 worldMatrix;

void main()
{
  gl_Position = viewProjectionMatrix * worldMatrix * vec4( position, 1.0 );
}

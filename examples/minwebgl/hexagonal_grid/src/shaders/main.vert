#version 300 es

layout( location = 0 ) in vec2 point;

uniform mat4 MVP;

void main()
{
  vec4 position = MVP * vec4( point, 0.0, 1.0 );
  gl_Position = position;
}

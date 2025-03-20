#version 300 es

layout( location = 0 ) in vec2 point;

void main()
{
  gl_Position = vec4( point, 0.0, 1.0 );
}

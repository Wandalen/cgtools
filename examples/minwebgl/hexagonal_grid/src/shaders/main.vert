#version 300 es

layout( location = 0 ) in vec2 point;

uniform mat3 MVP;

void main()
{
  vec3 pos3 = MVP * vec3( point, 1.0 );
  vec4 pos4 = vec4( pos3.x, pos3.y, 0.0, 1.0 );
  gl_Position = pos4;
}

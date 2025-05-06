#version 300 es

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec2 translation;
uniform mat3 u_mvp;
uniform mat3 u_rotation;

void main()
{
  vec3 pos = u_rotation * vec3( position, 1.0 );
  pos.xy += translation;
  pos = u_mvp * pos;
  vec4 pos4 = vec4( pos.xy, 0.0, 1.0 );
  gl_Position = pos4;
}

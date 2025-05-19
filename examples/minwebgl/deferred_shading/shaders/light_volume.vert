#version 300 es

layout( location = 0 ) in vec3 a_position;
layout( location = 1 ) in vec3 a_translation;
layout( location = 2 ) in float a_scale;

flat out mediump vec3 v_light_position;
flat out mediump float v_light_radius;

uniform mat4 u_mvp;

void main()
{
  v_light_position = a_translation;
  v_light_radius = a_scale;
  gl_Position = u_mvp * vec4( a_position * a_scale + a_translation, 1.0 );
}

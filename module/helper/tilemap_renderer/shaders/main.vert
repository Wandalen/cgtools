#version 300 es

layout( location = 0 ) in vec2 a_position;
layout( location = 1 ) in vec2 a_translation;
layout( location = 2 ) in vec2 a_rotation_cos_sin;
layout( location = 3 ) in vec2 a_scale;

uniform vec2 u_aspect_scale;

void main()
{
  mat2 rot = mat2
  (
    vec2( a_rotation_cos_sin.x, a_rotation_cos_sin.y ),
    vec2( -a_rotation_cos_sin.y, a_rotation_cos_sin.x )
  );
  vec2 pos = ( rot * ( a_scale * a_position ) + a_translation ) * u_aspect_scale;
  gl_Position = vec4( pos, 0.0, 1.0 );
}

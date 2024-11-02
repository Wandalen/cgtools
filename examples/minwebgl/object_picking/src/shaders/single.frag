#version 300 es

precision mediump float;

layout( location = 0 ) out vec4 frag_color;
layout( location = 1 ) out int id;

in vec3 v_normal;
in vec3 v_frag_pos;
in vec2 v_texcoord;
flat in int v_id;

uniform sampler2D u_diffuse;

const float AMBIENT = 0.4;
const vec3 DIRECTIONAL_LIGHT = vec3( -1.0, -1.0, -1.0 );
const vec3 POINT_LIGHT_POSITION = vec3( 0.0, 0.0, 0.0 );
const vec3 LIGHT_COLOR = vec3( 1.0, 1.0, 1.0 );

void main()
{
  vec3 normal = normalize( v_normal );

  float directional = max( dot( normal, normalize( -DIRECTIONAL_LIGHT ) ), 0.0 );

  vec3 offset = POINT_LIGHT_POSITION - v_frag_pos;
  vec3 direction = normalize( offset );
  float point = max( dot( normal, direction ), 0.0 );

  vec4 light = vec4( LIGHT_COLOR * ( directional + point + AMBIENT ), 1.0 );

  frag_color = texture( u_diffuse, v_texcoord ) * light;
  id = v_id;
}

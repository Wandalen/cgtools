#version 300 es

precision mediump float;

in vec3 v_world_pos;
in vec3 v_normal;
in vec2 v_texcoord;

uniform vec3 u_camera_pos;
uniform sampler2D u_ltc1;
uniform sampler2D u_ltc2;
uniform float u_roughness;

const float LUT_SIZE = 64.0;
const float LUT_SCALE = ( LUT_SIZE - 1.0 ) / LUT_SIZE;
const float LUT_BIAS = 0.5 / LUT_SIZE;

void main()
{
  vec3 normal = normalize( v_normal );
  vec3 view_dir = normalize( u_camera_pos - v_world_pos );
  float halfway = max( dot( normal, view_dir ), 0.0 );
  vec2 texcoord = vec2( u_roughness, sqrt( 1.0 - halfway ) );
  texcoord = texcoord * LUT_SCALE + LUT_BIAS;
  vec4 t1 = texture( u_ltc1, texcoord );
  vec4 t2 = texture( u_ltc2, texcoord );
  mat3 m = mat3
  (
    vec3( t1.x, 0.0, t1.y ),
    vec3( 0.0,  1.0, 0.0  ),
    vec3( t1.z, 0.0, t1.w )
  );

}

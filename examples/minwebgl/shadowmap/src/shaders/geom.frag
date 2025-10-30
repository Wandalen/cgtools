#version 300 es
precision highp float;
precision highp sampler2D;

in vec3 v_world_pos;
in vec3 v_normal;
in vec4 v_light_space_pos;

uniform vec3 u_light_dir;
uniform vec3 u_view_pos;
uniform vec3 u_light_color;
uniform vec3 u_object_color;
uniform sampler2D u_shadow_map;

out vec4 frag_color;

float inverse_lerp( float v, float min_value, float max_value )
{
  return ( v - min_value ) / ( max_value  - min_value );
}

float remap( float v, float in_min, float in_max, float out_min, float out_max )
{
  float t = inverse_lerp( v, in_min, in_max );
  return mix( out_min, out_max, t );
}

void main()
{
  // Calculate normalized surface normal (needed for both lighting and shadow bias)
  vec3 norm = normalize( v_normal );

  float shadow = 0.0;

  // Diffuse (use actual light direction from shadow frustum)
  vec3 light_dir_diffuse = -u_light_dir;
  float diff = max( dot( norm, light_dir_diffuse ), 0.0 );
  vec3 diffuse = diff * u_light_color;

  // Hemi-light
  vec3 sky_color = vec3( 0.0, 0.2, 0.4 );
  vec3 ground_color = vec3( 0.1, 0.05, 0.0 );
  float hemi_mix = remap( norm.y, -1.0, 1.0, 0.0, 1.0 );
  vec3 hemi = mix( ground_color, sky_color, hemi_mix );

  // Specular (Blinn-Phong)
  float specular_strength = 0.1;
  vec3 view_dir = normalize( u_view_pos - v_world_pos );
  vec3 halfway_dir = normalize( light_dir_diffuse + view_dir );
  float spec = pow( max( dot( norm, halfway_dir ), 0.0 ), 4.0 );
  vec3 specular = specular_strength * spec * u_light_color;

  // Apply shadow (only affects direct lighting, not hemi)
  vec3 result = ( hemi + ( diffuse + specular ) * ( 1.0 - shadow ) ) * u_object_color;
  frag_color = vec4( pow( result, vec3( 1.0 / 2.2 ) ), 1.0 );
}

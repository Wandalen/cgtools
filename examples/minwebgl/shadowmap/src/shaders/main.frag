#version 300 es
precision highp float;
precision highp sampler2D;

in vec3 v_world_pos;
in vec3 v_normal;
in vec2 v_texcoord;

uniform vec3 u_light_pos;
uniform vec3 u_view_pos;
uniform vec3 u_light_color;
uniform vec3 u_object_color;
uniform sampler2D u_lightmap;

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
  // Calculate normalized surface normal
  vec3 norm = normalize( v_normal );

  // Sample pre-baked shadow from lightmap
  float shadow = texture( u_lightmap, v_texcoord ).r;

  // === Point Light Calculation ===
  // Calculate light direction and distance for point light
  vec3 light_vec = u_light_pos - v_world_pos;
  float light_distance = length( light_vec );
  vec3 light_dir = light_vec / light_distance;  // Normalize

  // Distance attenuation (quadratic falloff)
  // attenuation = 1.0 / (constant + linear * d + quadratic * d^2)
  const float constant_att = 1.0;
  const float linear_att = 0.014;
  const float quadratic_att = 0.0007;
  float attenuation = 1.0 / ( constant_att + linear_att * light_distance + quadratic_att * light_distance * light_distance );

  // Diffuse lighting (Lambert)
  float diff = max( dot( norm, light_dir ), 0.0 );
  vec3 diffuse = diff * u_light_color * attenuation;

  // Hemi-light (ambient)
  vec3 sky_color = vec3( 0.0, 0.2, 0.4 );
  vec3 ground_color = vec3( 0.1, 0.05, 0.0 );
  float hemi_mix = remap( norm.y, -1.0, 1.0, 0.0, 1.0 );
  vec3 hemi = mix( ground_color, sky_color, hemi_mix );

  // Specular lighting (Blinn-Phong)
  float specular_strength = 0.1;
  vec3 view_dir = normalize( u_view_pos - v_world_pos );
  vec3 halfway_dir = normalize( light_dir + view_dir );
  float spec = pow( max( dot( norm, halfway_dir ), 0.0 ), 4.0 );
  vec3 specular = specular_strength * spec * u_light_color * attenuation;

  // float s = pow( 1.0 - shadow, 2.2 );
  float s = ( 1.0 - shadow );
  vec3 result = ( hemi + ( diffuse + specular ) * ( s ) ) * u_object_color;

  // Gamma correction
  frag_color = vec4( pow( result, vec3( 1.0 / 2.2 ) ), 1.0 );
}

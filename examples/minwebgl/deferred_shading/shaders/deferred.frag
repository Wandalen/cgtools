#version 300 es

precision mediump float;

flat in vec3 v_light_position;
flat in float v_light_radius;

uniform vec2 u_screen_size;
uniform sampler2D u_positions;
uniform sampler2D u_normals;

layout( location = 0 ) out vec4 frag_color;

float calculate_attenuation( vec3 light_pos, vec3 frag_pos, float light_radius )
{
  float distance = length( light_pos - frag_pos );
  float attenuation = clamp( ( 1.0 - pow( distance / light_radius, 4.0 ) ), 0.0, 1.0 );
  return attenuation * attenuation;
}

void main()
{
  vec3 albedo = vec3( 0.3, 0.27, 0.2 );
  vec3 view_position = vec3( 0.0 );
  vec2 tex_coord = ( gl_FragCoord.xy - 0.5 ) / u_screen_size;

  vec3 frag_pos = texture( u_positions, tex_coord ).rgb;
  vec3 normal = texture( u_normals, tex_coord ).rgb;

  vec3 light_dir = normalize( v_light_position - frag_pos );
  vec3 view_dir = normalize( view_position - frag_pos );
  vec3 halfway_dir = normalize( light_dir + view_dir );

  float diffuse = max( dot( normal, light_dir ), 0.0 );
  float specular = pow( max( dot( normal, halfway_dir ), 0.1 ), 10.0 );

  float attenuation = calculate_attenuation( v_light_position, frag_pos, v_light_radius );
  vec3 color = vec3( specular + diffuse ) * albedo * attenuation;

  frag_color = vec4( pow( color, vec3( 1.0 / 2.2 ) ), 1.0 );
}

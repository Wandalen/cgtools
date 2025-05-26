#version 300 es

precision mediump float;

// per instance data
flat in vec3 v_light_position;
flat in float v_light_radius;
flat in vec3 v_light_color;

uniform vec2 u_screen_size;
uniform sampler2D u_positions;
uniform sampler2D u_normals;
uniform sampler2D u_colors;

layout( location = 0 ) out vec4 frag_color;

float square( float x )
{
  return x * x;
}

float attenuate_no_cusp( float distance, float radius, float max_intensity, float falloff )
{
  float s = distance / radius;
  float s2 = square( s );

  if ( s >= 1.0 )
  {
    return 0.0;
  }
  else
  {
    return max_intensity * square( 1.0 - s2 ) / ( 1.0 + falloff * s2 );
  }
}

float attenuate_cusp( float distance, float radius, float max_intensity, float falloff )
{
  float s = distance / radius;
  float s2 = square( s );

  if ( s >= 1.0 )
  {
    return 0.0;
  }
  else
  {
    return max_intensity * square( 1.0 - s2 ) / ( 1.0 + falloff * s );
  }
}

void main()
{
  vec3 view_position = vec3( 0.0 );
  vec2 tex_coord = ( gl_FragCoord.xy - 0.5 ) / u_screen_size;
  vec3 frag_pos = texture( u_positions, tex_coord ).xyz;
  vec3 normal = texture( u_normals, tex_coord ).xyz;
  vec3 color = texture( u_colors, tex_coord ).rgb;

  vec3 to_ligth = v_light_position - frag_pos;
  float distance = length( to_ligth );
  vec3 light_dir = to_ligth / distance;
  vec3 view_dir = normalize( view_position - frag_pos );
  vec3 halfway_dir = normalize( light_dir + view_dir );
  float diffuse = max( dot( normal, light_dir ), 0.0 );
  float specular = pow( max( dot( normal, halfway_dir ), 0.0 ), 30.0 );
  float attenuation = attenuate_no_cusp( distance, v_light_radius, 2.0, 40.0 );
  vec3 colorr = vec3( specular + diffuse ) * color * attenuation;

  frag_color = vec4( pow( colorr, vec3( 1.0 / 2.2 ) ), 1.0 );
}

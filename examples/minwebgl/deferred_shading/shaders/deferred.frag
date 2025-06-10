#version 300 es

precision mediump float;

// per instance data
flat in vec3 v_light_position;
flat in float v_light_radius;
flat in vec3 v_light_color;

uniform vec2 u_screen_size;
uniform vec3 u_camera_position;
uniform sampler2D u_positions;
uniform sampler2D u_normals;
uniform sampler2D u_colors;

layout( location = 0 ) out vec4 frag_color;

float square( float x )
{
  return x * x;
}

// Attenuation function from this resource https://lisyarus.github.io/blog/posts/point-light-attenuation.html
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
  // Default Blinn-Phong shading model
  vec2 tex_coord = ( gl_FragCoord.xy - 0.5 ) / u_screen_size;
  vec3 frag_pos = texture( u_positions, tex_coord ).xyz;

  vec3 to_ligth = v_light_position - frag_pos;
  float distance = length( to_ligth );

  if ( distance > v_light_radius )
  {
    discard;
  }

  vec3 normal = texture( u_normals, tex_coord ).xyz;
  vec3 base_color = texture( u_colors, tex_coord ).rgb;

  vec3 light_dir = to_ligth / distance;
  vec3 view_dir = normalize( u_camera_position - frag_pos );
  vec3 halfway_dir = normalize( light_dir + view_dir );
  float diffuse = max( dot( normal, light_dir ), 0.0 );

  float specular_intensity = 0.5;
  float shininess = 32.0;
  float specular = specular_intensity * pow( max( dot( normal, halfway_dir ), 0.0 ), shininess );

  float attenuation = attenuate_cusp( distance, v_light_radius, 2.5, 4.0 );
  vec3 color = vec3( specular + diffuse ) * base_color * v_light_color * attenuation;

  frag_color = vec4( color, 1.0 );
}

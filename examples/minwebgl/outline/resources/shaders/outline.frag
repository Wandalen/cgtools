#version 300 es
// High precision is recommended for coordinate/distance calculations.
precision highp float;
// Interpolated texture coordinate from the vertex shader for the current pixel.
in vec2 v_tex_coord;
// Output fragment color to the default framebuffer ( screen ).
out vec4 FragColor;
// Input: The skybox equirectangular map
uniform sampler2D u_equirect_map;
// Input: The texture containing the original rendered object silhouette.
uniform sampler2D u_object_texture;
// Input: The final JFA result texture ( contains nearest seed coordinates for all pixels ).
uniform sampler2D u_jfa_texture;
// Uniforms for parameters needed for outlining.
uniform vec2 u_resolution;           // Screen/texture size in pixels
uniform float u_outline_thickness;   // Outline thickness in pixels
uniform vec4 u_outline_color;        // Color of the outline
uniform vec4 u_object_color;         // Fill color for the object itself
uniform mat4 u_inv_projection;          // Inverse projection matrix
uniform mat4 u_inv_view;                // Inverse view matrix

const float PI = 3.1415926535897932384626433;
const float FRAC_1_PI = 1.0 / PI;
const float FRAC_1_2PI = FRAC_1_PI / 2.0;

vec3 getWorldDir( vec2 uv )
{
  vec4 clip = vec4( uv * 2.0 - 1.0, -1.0, 1.0 );
  vec4 view = u_inv_projection * clip;
  view /= view.w;
  view.w = 0.0;
  vec3 worldDir = ( u_inv_view * view ).xyz;
  return normalize( worldDir );
}

vec2 dirToEquirectUV( vec3 dir )
{
  float phi = atan( dir.z, dir.x );
  float theta = asin( dir.y );
  vec2 uv = vec2( 0.5 + phi * FRAC_1_2PI, 0.5 - theta * FRAC_1_PI );

  return uv;
}

vec4 skybox()
{
  vec3 dir = getWorldDir( v_tex_coord );
  vec2 uv = dirToEquirectUV( dir );

  if ( uv.x > 0.001 && uv.x < 0.999 )
  {
    return texture( u_equirect_map, uv );
  }
  else
  {
    return texture( u_equirect_map, vec2( 0.0001, uv.y ) );
  }
}

void main()
{
  // Check if the current pixel belongs to the original object silhouette.
  // Sample the silhouette texture. Object pixels are white ( r=1.0 ).
  float object_present = texture(u_object_texture, v_tex_coord).r;

  if ( object_present > 0.01 ) // Use a small tolerance for float comparisons
  {
    // If the pixel is part of the object silhouette, draw it with the object color.
    FragColor = u_object_color;
  }
  else
  {
    // If the pixel is not part of the object ( it's background ), use the JFA result
    // to determine the distance to the nearest object pixel.

    // Sample the final JFA texture to get the coordinate of the nearest seed ( object pixel ).
    vec2 seed_coord = texture( u_jfa_texture, v_tex_coord ).xy;

    // Check if a valid seed coordinate was found ( i.e., not the sentinel value -1.0 ).
    // Assuming sentinel has x < 0.0.
    if ( seed_coord.x != 0.0 && seed_coord.y != 0.0 )
    {
        // Calculate the distance in pixel units between the current pixel and the nearest seed.
        // Scale normalized coordinates by resolution to get pixel coordinates.
        float dist = distance( v_tex_coord * u_resolution, seed_coord * u_resolution );

        // If the distance to the nearest object pixel is within the desired outline thickness...
        if ( dist < u_outline_thickness )
        {
          // ...draw the outline color.
          FragColor = u_outline_color;
        }
        else
        {
          // If the distance is greater than the outline thickness, draw the background color.
          FragColor = skybox();
        }
    }
    else
    {
      // If the sampled JFA coordinate was the sentinel ( -1.0, -1.0 ), it means
      // the JFA process didn't find any seed ( object pixel ) nearby within the
      // maximum jump distance. This pixel is far background.
      FragColor = skybox();
    }
  }
}

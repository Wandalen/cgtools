#version 300 es
// High precision is recommended for coordinate/distance calculations.
precision highp float;
// Interpolated texture coordinate from the vertex shader for the current pixel.
in vec2 v_tex_coord;
// Interpolated direction for sampling skybox texture
in vec3 v_dir;
// Output fragment color to the default framebuffer ( screen ).
out vec4 frag_color;
// Input: The skybox equirectangular map
uniform sampler2D u_equirect_map;
// Input: The texture containing the original rendered object silhouette.
uniform sampler2D u_object_texture;
// Input: The texture containing world-space normals for reflection calculation.
uniform sampler2D u_normal_texture;
// Input: The final JFA result texture ( contains nearest seed coordinates for all pixels ).
uniform sampler2D u_jfa_texture;
// Uniforms for parameters needed for outlining.
uniform vec2 u_resolution;           // Screen/texture size in pixels
uniform float u_outline_thickness;   // Outline thickness in pixels
uniform vec4 u_outline_color;        // Color of the outline
uniform vec4 u_object_color;
uniform mat4 u_inv_projection;          // Inverse projection matrix
uniform mat4 u_inv_view;                // Inverse view matrix

const float PI = 3.141592653589793;
const float INV_PI = 1.0 / PI;
const float INV_2PI = 1.0 / ( 2.0 * PI );

vec2 dirToEquirectUV( vec3 dir )
{
  vec3 d = normalize( dir );
  float phi = atan( d.z, d.x );
  float theta = asin( d.y );
  return vec2( 0.5 + phi * INV_2PI, theta * INV_PI + 0.5 );
}

vec4 skybox()
{
  vec2 uv = dirToEquirectUV( v_dir );

  return texture( u_equirect_map, uv );
}

vec4 sampleReflection()
{
  // Sample the world-space normal from the normal texture
  // Normals are stored in [0,1] range, convert back to [-1,1]
  vec3 normal = texture( u_normal_texture, v_tex_coord ).xyz;
  normal = normalize( normal );

  // Calculate the reflection direction
  // reflect() expects the incident vector (pointing towards the surface)
  vec3 reflectionDir = reflect( v_dir, normal );

  // Convert reflection direction to equirectangular UV coordinates
  vec2 uv = dirToEquirectUV( reflectionDir );

  // Sample the environment map
  return texture( u_equirect_map, uv );
}

void main()
{
  // Check if the current pixel belongs to the original object silhouette.
  // Sample the silhouette texture. Object pixels are white ( r=1.0 ).
  float object_present = texture( u_object_texture, v_tex_coord ).r;

  if ( object_present > 0.01 ) // Use a small tolerance for float comparisons
  {
    // If the pixel is part of the object silhouette, draw it with environment reflections.
    frag_color = u_object_color * sampleReflection();
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
          frag_color = u_outline_color;
        }
        else
        {
          // If the distance is greater than the outline thickness, draw the background color.
          frag_color = skybox();
        }
    }
    else
    {
      // If the sampled JFA coordinate was the sentinel ( -1.0, -1.0 ), it means
      // the JFA process didn't find any seed ( object pixel ) nearby within the
      // maximum jump distance. This pixel is far background.
      frag_color = skybox();
    }
  }
}

#version 300 es

precision highp float;

in vec2 v_tex_coord;
out vec4 FragColor;

// Defines the level of detail for edge detection. Higher values result in more prominent outlines.
const float DETAILS = 256.0;

// G-Buffer textures
uniform sampler2D u_depth_texture;
uniform sampler2D u_color_texture;

// Projection matrix for converting view-space coordinates to clip-space.
uniform mat4 u_projection;
// Resolution of the viewport, used for calculating pixel offsets.
uniform vec2 u_resolution;

uniform float u_outline_thickness;
uniform vec4 u_outline_color;
uniform vec4 u_object_color;
uniform vec4 u_background_color;

void main()
{
  // Sample the depth from the depth texture and convert it to a linear depth value.
  float depth = texture( u_depth_texture, v_tex_coord ).x;
  vec3 ndc = vec3( v_tex_coord, depth ) * 2.0 - 1.0;
  vec4 view = inverse( u_projection ) * vec4( ndc, 1.0 );
  view.xyz /= view.w;
  float linear_depth = - view.z;

  // Sample the color texture in a 3x3 kernel around the current fragment to perform edge detection.
  float pix[ 9 ];
  for( int y = 0; y < 3; y++ )
  {
    for( int x = 0; x < 3; x++ )
    {
      pix[ y * 3 + x ] = texture(
        u_color_texture,
        v_tex_coord + vec2( float( x - 1 ), float( y - 1 ) ) * u_outline_thickness / u_resolution
      ).r;
    }
  }

  // Apply Sobel operator to detect edges based on color differences.
  float sobel_src_x =
  (
    pix[0] * -1.0
    + pix[3] * -2.0
    + pix[6] * -1.0
    + pix[2] * 1.0
    + pix[5] * 2.0
    + pix[8] * 1.0
  );
  float sobel_src_y =
  (
    pix[0] * -1.0
    + pix[1] * -2.0
    + pix[2] * -1.0
    + pix[6] * 1.0
    + pix[7] * 2.0
    + pix[8] * 1.0
  );
  float sobel = length( vec2( sobel_src_x, sobel_src_y ) );

  // Calculate the outline intensity, scaled by depth to reduce outlines on distant objects.
  float outline = 1.0 - sobel * DETAILS * linear_depth / 8.0;

  // Clamp the outline value to ensure it's within a valid range [0, 1].
  outline = clamp( outline, 0.0, 1.0 );

  // Determine the final fragment color based on sampled color and calculated outline.
  if ( texture( u_color_texture, v_tex_coord ).x > 0.1 )
  {
    FragColor = u_object_color;
  }
  else if ( outline < 0.5 )
  {
    FragColor = u_outline_color;
  }
  else
  {
    FragColor = u_background_color;
  }
}
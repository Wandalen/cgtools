#version 300 es

precision highp float;

in vec2 v_tex_coord;
out vec4 FragColor;

const uint IDS[ 13 ] = uint[ 13 ](
  2u,
  6u,
  7u,
  8u,
  10u,
  11u,
  12u,
  13u,
  14u,
  16u,
  17u,
  18u,
  22u
);

// G-Buffer textures
uniform sampler2D u_color_texture;
uniform sampler2D u_depth_texture;
uniform sampler2D u_norm_texture;

// Projection matrix for converting view-space coordinates to clip-space.
uniform mat4 u_projection;
// Resolution of the viewport, used for calculating pixel offsets.
uniform vec2 u_resolution;

uniform float u_outline_thickness;
uniform vec4 u_background_color;

float outline_stencil_normal()
{
  // Sample the color texture in a 3x3 kernel around the current fragment to perform edge detection.
  float pix[ 25 ];
  for( int y = 0; y < 5; y++ )
  {
    for( int x = 0; x < 5; x++ )
    {
      pix[ y * 5 + x ] = length
      (
        texture(
          u_norm_texture,
          v_tex_coord + vec2( float( x - 2 ), float( y - 2 ) ) * u_outline_thickness / u_resolution
        )
      );
    }
  }

  float laplacian =
  (
    + pix[ 2 ] * -1.0
    + pix[ 6 ] * -2.0
    + pix[ 7 ] * -4.0
    + pix[ 8 ] * -2.0
    + pix[ 10 ] * -1.0
    + pix[ 11 ] * -4.0
    + pix[ 12 ] * 28.0
    + pix[ 13 ] * -4.0
    + pix[ 14 ] * -1.0
    + pix[ 16 ] * -2.0
    + pix[ 17 ] * -4.0
    + pix[ 18 ] * -2.0
    + pix[ 22 ] * -1.0
  );

  // Clamp the outline value to ensure it's within a valid range [0, 1].
  float outline = clamp( laplacian, 0.0, 1.0 );

  return outline;
}

float outline_stencil_depth()
{
  // Sample the color texture in a 3x3 kernel around the current fragment to perform edge detection.
  float pix[ 25 ];
  for( int y = 0; y < 5; y++ )
  {
    for( int x = 0; x < 5; x++ )
    {
      pix[ y * 5 + x ] = length
      (
        texture(
          u_depth_texture,
          v_tex_coord + vec2( float( x - 2 ), float( y - 2 ) ) * u_outline_thickness / u_resolution
        )
      );
    }
  }

  float laplacian =
  (
    + pix[ 2 ] * -1.0
    + pix[ 6 ] * -2.0
    + pix[ 7 ] * -4.0
    + pix[ 8 ] * -2.0
    + pix[ 10 ] * -1.0
    + pix[ 11 ] * -4.0
    + pix[ 12 ] * 28.0
    + pix[ 13 ] * -4.0
    + pix[ 14 ] * -1.0
    + pix[ 16 ] * -2.0
    + pix[ 17 ] * -4.0
    + pix[ 18 ] * -2.0
    + pix[ 22 ] * -1.0
  );

  // Clamp the outline value to ensure it's within a valid range [0, 1].
  float outline = clamp( laplacian, 0.0, 1.0 );

  return outline;
}

float outline_stencil_color()
{
  float depth = 1.0 - texture( u_depth_texture, v_tex_coord ).x;

  // Sample the color texture in a 3x3 kernel around the current fragment to perform edge detection.
  float pix[ 25 ];
  for( int y = 0; y < 5; y++ )
  {
    for( int x = 0; x < 5; x++ )
    {
      pix[ y * 5 + x ] = texture
      (
        u_color_texture,
        v_tex_coord + vec2( float( x - 2 ), float( y - 2 ) ) * u_outline_thickness / u_resolution
      ).r;
    }
  }

  float laplacian =
  (
    + pix[ 2 ] * -1.0
    + pix[ 6 ] * -2.0
    + pix[ 7 ] * -4.0
    + pix[ 8 ] * -2.0
    + pix[ 10 ] * -1.0
    + pix[ 11 ] * -4.0
    + pix[ 12 ] * 28.0
    + pix[ 13 ] * -4.0
    + pix[ 14 ] * -1.0
    + pix[ 16 ] * -2.0
    + pix[ 17 ] * -4.0
    + pix[ 18 ] * -2.0
    + pix[ 22 ] * -1.0
  );

  // Clamp the outline value to ensure it's within a valid range [0, 1].
  float outline = clamp( laplacian, 0.0, 1.0 );

  return outline;
}

vec4 outline_color()
{
  vec4 near_color = vec4( 0.0 );
  float near_depth = 0.0;

  vec4 colors[ 25 ];
  for( int y = 0; y < 5; y++ )
  {
    for( int x = 0; x < 5; x++ )
    {
      colors[ y * 5 + x ] = texture
      (
        u_color_texture,
        v_tex_coord + vec2( float( x - 2 ), float( y - 2 ) ) * u_outline_thickness / u_resolution
      );
    }
  }

  float depths[ 25 ];
  for( int y = 0; y < 5; y++ )
  {
    for( int x = 0; x < 5; x++ )
    {
      depths[ y * 5 + x ] = 1.0 - texture
      (
        u_depth_texture,
        v_tex_coord + vec2( float( x - 2 ), float( y - 2 ) ) * u_outline_thickness / u_resolution
      ).r;
    }
  }

  for ( int i = 0; i < 13; i++ )
  {
    uint j = IDS[ i ];
    if ( near_depth < depths[ j ] && depths[ j ] < 1.0 )
    {
      near_depth = depths[ j ];
      near_color = colors[ j ];
    }
  }

  return near_color;
}

void main()
{
  float outline_s_color = outline_stencil_color();

  // Determine the final fragment color based on sampled color and calculated outline.
  if ( texture( u_color_texture, v_tex_coord ).x > 0.1 )
  {
    if ( outline_stencil_depth() > 0.001 )
    {
      FragColor = outline_color();
    }
    else if ( outline_stencil_normal() > 0.3 )
    {
      FragColor = vec4(1.0); //outline_color();
    }
    else
    {
      FragColor = u_background_color;
    }
  }
  else if ( outline_s_color < 0.5 )
  {
    FragColor = outline_color();
  }
  else
  {
    FragColor = u_background_color;
  }
}

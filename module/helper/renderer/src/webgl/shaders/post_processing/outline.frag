#define MAX_OBJECT_COUNT 1024

#version 300 es

precision highp float;

in vec2 vUv;
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

uniform sampler2D sourceTexture;

// G-Buffer textures
uniform sampler2D objectIdTexture;
uniform sampler2D depthTexture;

layout( std140 ) uniform ObjectColorBlock
{
  vec4 objectColors[ MAX_OBJECT_COUNT ];
};

// Projection matrix for converting view-space coordinates to clip-space.
uniform mat4 projection;
// Resolution of the viewport, used for calculating pixel offsets.
uniform vec2 resolution;
uniform float outlineThickness;

float outlineStencil() 
{
  // Sample the color texture in a 3x3 kernel around the current fragment to perform edge detection.
  float pickedObjectColors[ 25 ];
  for( int y = 0; y < 5; y++ )
  {
    for( int x = 0; x < 5; x++ )
    {
      uint objectId = uint( texture(
        objectIdTexture,
        vUv + vec2( float( x - 2 ), float( y - 2 ) ) * outlineThickness / resolution
      ).r );
      pickedObjectColors[ y * 5 + x ] = length( objectColors[ objectId ] );
    }
  }

  float laplacian =
  (
    + pickedObjectColors[ 2 ] * -1.0
    + pickedObjectColors[ 6 ] * -2.0
    + pickedObjectColors[ 7 ] * -4.0
    + pickedObjectColors[ 8 ] * -2.0
    + pickedObjectColors[ 10 ] * -1.0
    + pickedObjectColors[ 11 ] * -4.0
    + pickedObjectColors[ 12 ] * 28.0
    + pickedObjectColors[ 13 ] * -4.0
    + pickedObjectColors[ 14 ] * -1.0
    + pickedObjectColors[ 16 ] * -2.0
    + pickedObjectColors[ 17 ] * -4.0
    + pickedObjectColors[ 18 ] * -2.0
    + pickedObjectColors[ 22 ] * -1.0
  );

  // Clamp the outline value to ensure it's within a valid range [0, 1].
  float outline = clamp( laplacian, 0.0, 1.0 );

  return outline;
}

vec4 outlineColor()
{
  float depth = 1.0 - texture( depthTexture, vUv ).x;

  float nearObjectId = 0.0;
  float nearDepth = 0.0;

  vec4 objectIds[ 25 ];
  for( int y = 0; y < 5; y++ )
  {
    for( int x = 0; x < 5; x++ )
    {
      objectIds[ y * 5 + x ] = texture(
        objectIdTexture,
        vUv + vec2( float( x - 2 ), float( y - 2 ) ) * outlineThickness / resolution
      ).r;
    }
  }

  float depths[ 25 ];
  for( int y = 0; y < 5; y++ )
  {
    for( int x = 0; x < 5; x++ )
    {
      depths[ y * 5 + x ] = texture(
        depthTexture,
        vUv + vec2( float( x - 2 ), float( y - 2 ) ) * outlineThickness / resolution
      ).r;
    }
  }

  for ( int i = 0; i < 13; i++ )
  {
    uint j = IDS[ i ];
    if ( nearDepth < depths[ j ] && depths[ j ] >= 0.0 )
    {
      nearDepth = depths[ j ];
      nearObjectId = objectIds[ j ];
    }
  }

  return objectColors[ uint( nearObjectId ) ];
}

void main()
{
  float outline = outlineStencil();

  // Determine the final fragment color based on sampled color and calculated outline.
  if ( texture( sourceTexture, vUv ).x >= 0.0 )
  {
    FragColor = texture( sourceTexture, vUv );
  }
  else if ( outline < 0.9 )
  {
    FragColor = outlineColor();
  }
  else
  {
    FragColor = texture( sourceTexture, vUv );
  }
}
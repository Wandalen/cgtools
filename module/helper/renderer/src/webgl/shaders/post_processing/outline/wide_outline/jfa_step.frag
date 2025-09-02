#version 300 es
// High precision is recommended for coordinate/distance calculations.
precision highp float;
// Interpolated texture coordinate from the vertex shader for the current pixel.
in vec2 vUv;
// Output color/data for the JFA texture for the *next* step. Stores the nearest seed coordinate found.
out vec4 FragColor; // Outputting vec4 for RGBA32F texture
// Input: The JFA texture from the previous step ( contains nearest seed coordinates found so far ).
uniform sampler2D jfaTexture;
// Uniforms providing screen resolution and the current jump distance.
uniform vec2 resolution; // Screen/texture size in pixels
uniform vec2 stepSize;  // Current jump distance in pixels

void main()
{
  // Initialize the best distance found so far to a very large value
  float bestDistance = 1e20;
  // Initialize the best coordinate found so far to a sentinel value ( -1.0, -1.0 )
  vec2 bestCoord = vec2( -1.0 );

  // Loop through a 3x3 neighborhood centered around the current pixel.
  for ( int y = -1; y <= 1; ++y )
  {
    for ( int x = -1; x <= 1; ++x )
    {
      // Calculate the sampling offset in normalized texture coordinates.
      // The offset vector ( x, y ) is scaled by the current jump size ( stepSize )
      // and then divided by the resolution to convert from pixel space to normalized ( 0-1 ) texture space.
      // `ceil` is used to ensure step sizes are rounded up to the nearest pixel distance,
      // as the step size might not be a perfect integer during intermediate steps.
      vec2 offset = ceil( vec2( float( x ), float( y ) ) * stepSize ) / resolution;

      // Calculate the sample coordinate in the input JFA texture.
      vec2 sampleCoord = vUv + offset;

      // Sample the input JFA texture at the calculated sample coordinate.
      vec2 seedCoord = texture( jfaTexture, sampleCoord ).xy;

      // Calculate the distance between the current pixel's coordinate and the sampled seed coordinate.
      float dist = distance( vUv * resolution, seedCoord * resolution );

      if ( dist < bestDistance )
      {
        bestDistance = dist;
        bestCoord = seedCoord;
      }
    }
  }

  // Output the best ( nearest ) seed coordinate found in the neighborhood for this pixel.
  // This value will be used as input for the next JFA step pass.
  FragColor = vec4( bestCoord, 0.0, 1.0 );
}
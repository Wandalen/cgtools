#version 300 es

// High precision is recommended for coordinate/distance calculations.
precision highp float;

// Interpolated texture coordinate from the vertex shader for the current pixel.
in vec2 vUv;
// Output fragment color to the default framebuffer ( screen ).
out vec4 FragColor;

const float outlineThickness = 30.0;   // Outline thickness in pixels

uniform sampler2D sourceTexture;
// Input: The texture containing the original rendered object silhouette.
uniform sampler2D objectColorTexture;
// Input: The final JFA result texture ( contains nearest seed coordinates for all pixels ).
uniform sampler2D jfaTexture;
// Uniforms for parameters needed for outlining.
uniform vec2 resolution;           // Screen/texture size in pixels

void main()
{
  // Check if the current pixel belongs to the original object silhouette.
  // Sample the silhouette texture. Object pixels are white ( r=1.0 ).
  float objectPresent = texture( objectColorTexture, vUv ).r;

  if ( objectPresent > 0.01 ) // Use a small tolerance for float comparisons
  {
    // If the pixel is part of the object silhouette, draw it with the object color.
    FragColor = texture( sourceTexture, vUv );
  }
  else
  {
    // If the pixel is not part of the object ( it's background ), use the JFA result
    // to determine the distance to the nearest object pixel.

    // Sample the final JFA texture to get the coordinate of the nearest seed ( object pixel ).
    vec2 seedCoord = texture( jfaTexture, vUv ).xy;

    // Check if a valid seed coordinate was found ( i.e., not the sentinel value -1.0 ).
    // Assuming sentinel has x < 0.0.
    if ( seedCoord.x != 0.0 && seedCoord.y != 0.0 )
    {
        // Calculate the distance in pixel units between the current pixel and the nearest seed.
        // Scale normalized coordinates by resolution to get pixel coordinates.
        float dist = distance( vUv * resolution, seedCoord * resolution );

        // If the distance to the nearest object pixel is within the desired outline thickness...
        if ( dist < outlineThickness )
        {
          // ...draw the outline color.
          FragColor = texture( objectColorTexture, vUv );
        }
        else
        {
          // If the distance is greater than the outline thickness, draw the background color.
          FragColor = texture( sourceTexture, vUv );
        }
    }
    else
    {
      // If the sampled JFA coordinate was the sentinel ( -1.0, -1.0 ), it means
      // the JFA process didn't find any seed ( object pixel ) nearby within the
      // maximum jump distance. This pixel is far background.
      FragColor = texture( sourceTexture, vUv );
    }
  }
}
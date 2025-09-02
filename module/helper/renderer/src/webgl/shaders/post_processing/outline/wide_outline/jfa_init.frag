#version 300 es
// High precision is recommended for coordinate/distance calculations.
precision highp float;
// Interpolated texture coordinate from the vertex shader.
in vec2 vUv;
// Output color/data for the JFA texture. We store a vec2 ( coordinates ) in a vec4.
// RGBA32F texture format is typically used for this pass to store floating-point coordinates.
out vec4 FragColor;
// Input: The texture containing the rendered object silhouette ( from object_pass ).
uniform sampler2D objectColorTexture;

void main()
{
  // Check if the pixel corresponds to the object silhouette in the input texture.
  // The object_pass renders object pixels as white ( r=1.0 ).
  float objectPresent = texture( objectColorTexture, vUv ).r;

  if ( objectPresent > 0.01 ) // If pixel is part of the object ( check > 0.0 for robustness )
  {
    // These are the "seeds" for the JFA. Store the pixel's own normalized texture coordinates ( 0-1 ).
    // We store them in the first two components ( xy ) of the output vec4.
    FragColor = vec4( vUv, 0.0, 1.0 );
  } 
  else 
  {
    // Mark background pixels with a sentinel value. A common sentinel is ( -1.0, -1.0 ).
    // This indicates that no seed has been found for this pixel yet.
    FragColor = vec4( -1.0, -1.0, -1.0, 1.0 );
  }
}
#version 300 es
// NOTE : a near-identical sibling of this JFA shader lives at
// examples/minwebgl/outline/resources/shaders/jfa_init.frag ( naming differs : vUv here vs v_tex_coord there ).
// The duplication is intentional : that example is a self-contained minwebgl walkthrough of the JFA
// technique, while this copy is the production post-processing integration. Mirror JFA-core bug fixes.
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
  // Fix(BUG-181): `gbuffer.rs`'s `GBuffer::render` clears every color attachment, including
  // OBJECT_COLOR, to ( -1, -1, -1, 1 ) before drawing, and `gbuffer.frag` writes a real object
  // color to every rasterized object pixel ( `FragObjectColor = objectColor;` -- r/g/b always
  // move together, in both the clear and the write ). This used to check `> 0.01`, which only
  // matched objects whose red channel happened to be close to 1.0 ( true only by coincidence of
  // the one caller that existed at the time always using red ); any object with a different
  // color -- pure green/blue/cyan, or even ordinary black -- had `r <= 0.01` and was silently
  // treated as background, so that object never seeded the JFA and received no outline at all.
  // The red channel alone is still enough to test: any non-negative value can only come from a
  // real ( always non-negative ) object color, never from the negative sentinel clear value.
  float objectColorR = texture( objectColorTexture, vUv ).r;

  if ( objectColorR >= 0.0 ) // Any non-negative red channel means a real object color was written here.
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
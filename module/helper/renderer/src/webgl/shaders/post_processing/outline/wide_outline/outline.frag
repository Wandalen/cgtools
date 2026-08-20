#version 300 es

// High precision is recommended for coordinate/distance calculations.
precision highp float;

// Interpolated texture coordinate from the vertex shader for the current pixel.
in vec2 vUv;
// Output fragment color to the default framebuffer ( screen ).
out vec4 FragColor;

uniform sampler2D sourceTexture;
// Input: The texture containing the original rendered object silhouette.
uniform sampler2D objectColorTexture;
// Input: The final JFA result texture ( contains nearest seed coordinates for all pixels ).
uniform sampler2D jfaTexture;
// Uniforms for parameters needed for outlining.
uniform vec2 resolution;           // Screen/texture size in pixels
// Fix(BUG-179): this was a hardcoded `const float outlineThickness = 30.0;`, completely
// disconnected from `WideOutlinePass::outline_thickness` -- the JFA step pass alone consumed
// that field ( sizing its search radius ), but the actual draw/no-draw distance check below
// always compared against this fixed constant, so no caller-supplied thickness ever reached
// the pixel that decides whether to draw the outline.
uniform float outlineThickness;    // Outline thickness in pixels

void main()
{
  // Check if the current pixel belongs to the original object silhouette.
  // Fix(BUG-193): same defect as BUG-181's `jfa_init.frag` fix, duplicated here -- this checked
  // `> 0.01`, assuming object pixels are red-dominant, but `GBuffer::render` clears OBJECT_COLOR
  // to ( -1, -1, -1, 1 ) and `gbuffer.frag` writes the caller's arbitrary `objectColor` uniform
  // verbatim, so any object color with `r <= 0.01` ( pure green/blue/cyan, or black ) was
  // silently treated as background here too, drawing the plain source color over that object's
  // own pixels instead of its actual rendered appearance.
  float objectColorR = texture( objectColorTexture, vUv ).r;

  if ( objectColorR >= 0.0 ) // Any non-negative red channel means a real object color was written here.
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
    // Fix(BUG-182): this compared against `0.0` ( `!= 0.0` ), not the actual sentinel `-1.0` the
    // comment itself names -- `jfa_init.frag` writes `vec4(-1.0, -1.0, -1.0, 1.0)` for
    // no-seed-found pixels, and real seed coordinates are always non-negative UV values
    // ( `vec4(vUv, 0.0, 1.0)` ), so the correct discriminant is sign, not inequality with zero.
    // `!= 0.0` both accepted the real `(-1,-1)` sentinel as "valid" ( `-1.0 != 0.0` is true ) and
    // rejected any genuinely-found seed whose coordinate happened to land exactly on `0.0`.
    if ( seedCoord.x >= 0.0 && seedCoord.y >= 0.0 )
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
#version 300 es
precision highp float;

//! Color Grading Post-Processing Shader
//!
//! Implements standard color grading operations for cinematic look development.
//!
//! ## Technical References
//!
//! ### Luminance Calculation (line 121)
//! - **ITU-R Recommendation BT.709** (Rec. 709)
//!   Luma coefficients: Y = 0.2126*R + 0.7152*G + 0.0722*B
//!   Standard for HDTV color space and widely used in digital imaging
//!   https://www.itu.int/rec/R-REC-BT.709/
//!
//! ### Tone Curves (line 92)
//! - **Smoothstep S-curve**: f(x) = 3x² - 2x³
//!   Standard sigmoid function for smooth interpolation
//!   Provides filmic look with lifted shadows and rolled-off highlights
//!   Reference: Ken Perlin, "Improving Noise" (2002)
//!   https://mrl.cs.nyu.edu/~perlin/paper445.pdf
//!
//! ### White Balance
//! - Temperature/tint adjustment using simple RGB multipliers
//!   Common technique in digital photography and color grading
//!   Reference: "Digital Color Management" by Edward Giorgianni & Thomas Madden
//!
//! ### Vibrance vs Saturation
//! - **Saturation**: Uniform adjustment of all color channels (standard technique)
//! - **Vibrance**: Smart saturation affecting less-saturated colors more
//!   Preserves skin tones better than uniform saturation
//!   Implementation based on Adobe Photoshop's vibrance algorithm principles
//!
//! ### Exposure and Tonal Adjustments
//! - Exponential exposure: 2^exposure for natural brightness scaling
//! - Shadow/highlight masking: Squared masks for smooth falloff
//!   Common technique in HDR tone mapping and color grading
//!   Reference: Reinhard et al., "Photographic Tone Reproduction for Digital Images" (2002)
//!
//! ## Implementation Notes
//!
//! All parameters designed for -1.0 to 1.0 range with 0.0 as neutral.
//! Shader remaps these to appropriate internal ranges to prevent over-sensitivity.
//!
//! Techniques are standard industry practices combined with original parameter
//! mapping designed for this implementation.

uniform sampler2D sourceTexture;

// Color grading parameters (all designed for -1.0 to 1.0 range with 0.0 as neutral)
uniform float temperature;
uniform float tint;
uniform float exposure;
uniform float shadows;
uniform float highlights;
uniform float contrast;
uniform float vibrance;
uniform float saturation;

in vec2 vUv;
out vec4 frag_color;

vec3 apply_white_balance( vec3 color, float temperature, float tint )
{
  // White balance adjustment with stronger, more visible effect
  vec3 t = vec3( 1.0 );
  // Temperature: -1 = cool blue, 0 = neutral, 1 = warm orange (20% shift)
  t.r += 0.2 * temperature - 0.1 * tint;
  t.b -= 0.2 * temperature + 0.1 * tint;
  return color * t;
}

vec3 apply_tonal_adjustments( vec3 color, float exposure, float shadows, float highlights )
{
  // 1. Overall exposure adjustment (simple and predictable)
  if ( abs( exposure ) > 0.001 )
  {
    // Exponential scaling for natural brightness feel
    // -1 = half brightness, 0 = neutral, 1 = double brightness
    float exp_scale = pow( 2.0, exposure );
    color *= exp_scale;
  }

  // 2. Shadow adjustment (only affects dark areas)
  if ( abs( shadows ) > 0.001 )
  {
    // Create smooth mask: dark areas get more adjustment, bright areas get none
    vec3 shadow_mask = 1.0 - color; // Inverted: dark = 1, bright = 0
    shadow_mask = shadow_mask * shadow_mask; // Square for smoother falloff

    // Shadow lift: adds light to dark areas without blowing out
    // -1 = crush shadows darker, 0 = neutral, 1 = lift shadows brighter
    vec3 shadow_adjustment = shadow_mask * shadows * 0.15;
    color += shadow_adjustment;
  }

  // 3. Highlight adjustment (only affects bright areas)
  if ( abs( highlights ) > 0.001 )
  {
    // Create smooth mask: bright areas get more adjustment, dark areas get none
    vec3 highlight_mask = color; // Bright = high, dark = low
    highlight_mask = highlight_mask * highlight_mask; // Square for smoother falloff

    // Highlight recovery: compresses bright areas to recover detail
    // -1 = blow out highlights, 0 = neutral, 1 = recover/compress highlights
    if ( highlights > 0.0 )
    {
      // Positive: compress highlights (recovery)
      vec3 compressed = color / ( 1.0 + highlight_mask * highlights * 1.2 );
      color = mix( color, compressed, highlight_mask );
    }
    else
    {
      // Negative: expand highlights (blow out)
      vec3 expanded = color * ( 1.0 - highlight_mask * highlights * 0.6 );
      color = mix( color, expanded, highlight_mask );
    }
  }

  return max( color, vec3( 0.0 ) ); // Clamp to prevent negative values
}

vec3 apply_filmic_curve( vec3 color, float amount )
{
  // Cinematic tone curve that makes images look beautiful
  // Based on filmic response with lifted shadows and rolled-off highlights
  // -1 = flat/matte look, 0 = neutral (pass-through), 1 = cinematic with depth

  if ( abs( amount ) < 0.001 ) return color; // Fast path for neutral

  if ( amount > 0.0 )
  {
    // Positive: Cinematic look (lift shadows, compress highlights)
    // Use smoothstep-based S-curve: 3x^2 - 2x^3
    vec3 x = clamp( color, 0.0, 1.0 );
    vec3 s_curve = x * x * ( 3.0 - 2.0 * x );

    // Blend between linear and S-curve based on amount
    return mix( color, s_curve, amount * 0.5 );
  }
  else
  {
    // Negative: Flatten (matte look, reduced dynamic range)
    // Simply reduce the distance from midpoint
    const vec3 mid = vec3( 0.5 );
    vec3 diff = color - mid;
    float flatten = 1.0 + amount; // -1 gives 0 (full grey), 0 gives 1 (neutral)
    return mid + diff * flatten;
  }
}

vec3 adjust_vibrance( vec3 color, float vibrance )
{
  // Smart saturation that affects less-saturated colors more
  float average = ( color.r + color.g + color.b ) / 3.0;
  float mx = max( max( color.r, color.g ), color.b );
  float amt = ( mx - average ) * ( -vibrance * 3.0 );
  return mix( color, vec3( mx ), amt );
}

vec3 adjust_saturation( vec3 color, float saturation_param )
{
  // Remap: -1..1 -> 0.2..1.8 (centered at 1.0 for neutral)
  float saturation = 1.0 + saturation_param * 0.8;
  float luma = dot( color, vec3( 0.2126, 0.7152, 0.0722 ) );
  return mix( vec3( luma ), color, saturation );
}

void main()
{
  vec3 color = texture( sourceTexture, vUv ).rgb;

  color = apply_white_balance( color, temperature, tint );
  color = apply_tonal_adjustments( color, exposure, shadows, highlights );
  color = apply_filmic_curve( color, contrast );
  color = adjust_saturation( color, saturation );
  color = adjust_vibrance( color, vibrance );

  frag_color = vec4( clamp( color, 0.0, 1.0 ), 1.0 );
}

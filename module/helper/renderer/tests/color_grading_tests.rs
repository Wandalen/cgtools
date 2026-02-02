//! Tests for color grading functionality

use renderer::webgl::post_processing::ColorGradingParams;

#[ test ]
fn test_color_grading_params_default_values()
{
  let params = ColorGradingParams::default();

  // Verify all parameters return neutral (0.0) values by default
  assert_eq!( params.temperature, 0.0, "Default temperature should be 0.0 (neutral)" );
  assert_eq!( params.tint, 0.0, "Default tint should be 0.0 (neutral)" );
  assert_eq!( params.exposure, 0.0, "Default exposure should be 0.0 (neutral)" );
  assert_eq!( params.shadows, 0.0, "Default shadows should be 0.0 (neutral)" );
  assert_eq!( params.highlights, 0.0, "Default highlights should be 0.0 (neutral)" );
  assert_eq!( params.contrast, 0.0, "Default contrast should be 0.0 (neutral)" );
  assert_eq!( params.vibrance, 0.0, "Default vibrance should be 0.0 (neutral)" );
  assert_eq!( params.saturation, 0.0, "Default saturation should be 0.0 (neutral)" );
}

#[ test ]
fn test_color_grading_params_clone()
{
  let params = ColorGradingParams
  {
    temperature : 0.5,
    tint : -0.3,
    exposure : 0.2,
    shadows : 0.1,
    highlights : -0.1,
    contrast : 0.4,
    vibrance : 0.3,
    saturation : 0.2,
  };

  let cloned = params.clone();

  assert_eq!( cloned.temperature, 0.5 );
  assert_eq!( cloned.tint, -0.3 );
  assert_eq!( cloned.exposure, 0.2 );
  assert_eq!( cloned.shadows, 0.1 );
  assert_eq!( cloned.highlights, -0.1 );
  assert_eq!( cloned.contrast, 0.4 );
  assert_eq!( cloned.vibrance, 0.3 );
  assert_eq!( cloned.saturation, 0.2 );
}

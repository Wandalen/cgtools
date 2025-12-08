//! Integration tests related to EasingFunction and EasingBuilder
//! traits and structs that implements them

#[ cfg( test ) ]
mod tests
{
  use animation::easing::
  {
    base::{ EasingFunction, EasingBuilder },
    Linear, Step
  };
  use animation::easing::cubic::bezier::
  {
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack
  };

  #[ test ]
  fn test_linear_function()
  {
    // Linear easing should return the input value directly
    assert_eq!( Linear::new().apply( 0.0, 1.0, 0.5 ), 0.5 );
    assert_eq!( Linear::new().apply( 0.0, 1.0, 0.0 ), 0.0 );
    assert_eq!( Linear::new().apply( 0.0, 1.0, 1.0 ), 1.0 );
  }

  fn assert_f_eq( a : f64, b : f64, eps : f64 )
  {
    assert!( b - eps < a && a < b + eps );
  }

  #[ test ]
  fn test_step_function()
  {
    let eps = 0.001;
    // Step easing should progress in discrete steps
    let step_func = Step::new( 5.0 );
    assert_eq!( step_func.apply( 0.0, 1.0, 0.0 ), 0.0 );
    assert_f_eq( step_func.apply( 0.0, 1.0, 0.01 ), 0.2, eps );
    assert_f_eq(  step_func.apply( 0.0, 1.0, 0.2 ), 0.2, eps );
    assert_f_eq( step_func.apply( 0.0, 1.0, 0.21 ), 0.4, eps );
    assert_f_eq( step_func.apply( 0.0, 1.0, 0.4 ), 0.4, eps );
    assert_f_eq( step_func.apply( 0.0, 1.0, 0.81 ), 1.0, eps );
    assert_f_eq( step_func.apply( 0.0, 1.0, 1.0 ), 1.0, eps );
  }

  #[ test ]
  fn test_cubic_boundaries_and_properties()
  {
    // A list of all cubic easing functions to test common properties
    let cubic_functions : Vec< Box< dyn EasingFunction< AnimatableType = f32 > > > = vec!
    [
      EaseInSine::new(),
      EaseOutSine::new(),
      EaseInOutSine::new(),
      EaseInQuad::new(),
      EaseOutQuad::new(),
      EaseInOutQuad::new(),
      EaseInCubic::new(),
      EaseOutCubic::new(),
      EaseInOutCubic::new(),
      EaseInQuart::new(),
      EaseOutQuart::new(),
      EaseInOutQuart::new(),
      EaseInQuint::new(),
      EaseOutQuint::new(),
      EaseInOutQuint::new(),
      EaseInExpo::new(),
      EaseOutExpo::new(),
      EaseInOutExpo::new(),
      EaseInCirc::new(),
      EaseOutCirc::new(),
      EaseInOutCirc::new(),
      EaseInBack::new(),
      EaseOutBack::new(),
      EaseInOutBack::new(),
    ];

    // All cubic functions should return 0.0 at t = 0.0 and 1.0 at t = 1.0
    for easing_function in cubic_functions
    {
      assert_eq!( easing_function.apply( 0.0, 1.0, 0.0 ), 0.0, "{:?} should start at 0.0", easing_function );
      assert_eq!( easing_function.apply( 0.0, 1.0, 1.0 ), 1.0, "{:?} should end at 1.0", easing_function );
    }
  }

  #[ test ]
  fn test_back_easing_overshoot()
  {
    // Back easing functions should have values outside the [ 0.0, 1.0 ] range
    assert!( EaseInBack::new().apply( 0.0, 1.0, 0.1 ) < 0.0 );
    assert!( EaseOutBack::new().apply( 0.0, 1.0, 0.9 ) > 1.0 );
    assert!( EaseInOutBack::new().apply( 0.0, 1.0, 0.1 ) < 0.0 );
    assert!( EaseInOutBack::new().apply( 0.0, 1.0, 0.9 ) > 1.0 );
  }

  #[ test ]
  fn test_specific_easing_behaviors()
  {
    // EaseInQuad should be slower than linear at the start
    assert!( EaseInQuad::new().apply( 0.0, 1.0, 0.2 ) < Linear::new().apply( 0.0, 1.0, 0.2 ) );

    // EaseOutQuad should be faster than linear at the start
    assert!( EaseOutQuad::new().apply( 0.0, 1.0, 0.2 ) > Linear::new().apply( 0.0, 1.0, 0.2 ) );
  }
}

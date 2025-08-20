mod private
{ 
  use crate::easing::base::
  { 
    impl_easing_function,
    EasingFunction, 
    EasingBuilder 
  };

  pub struct Cubic
  {
    in_tangent : [ f32; 2 ],
    out_tangent : [ f32; 2 ],
    iterations : usize
  }

  impl Cubic
  {
    fn get_x( &self, t: f32 ) -> f32 
    {
      let one_minus_t = 1.0 - t;
      3.0 * one_minus_t.powi( 2 ) * t * self.in_tangent[ 0 ]
      + 3.0 * one_minus_t * t.powi( 2 ) * self.out_tangent[ 0 ]
      + t.powi( 3 )
    }

    fn get_y( &self, t : f32 ) -> f32 
    {
      let one_minus_t = 1.0 - t;
      3.0 * one_minus_t.powi( 2 ) * t * self.in_tangent[ 1 ]
      + 3.0 * one_minus_t * t.powi( 2 ) * self.out_tangent[ 1 ]
      + t.powi( 3 )
    }

    pub fn new( parameters : [ f32; 4 ] ) -> Self 
    {
      let [ i1, i2, o1, o2 ] = parameters;
      Self
      {
        in_tangent : [ i1, i2 ],
        out_tangent : [ o1, o2 ],
        iterations : 0
      }
    }

    pub fn set_iterations( &mut self, iterations : usize ) 
    {
      self.iterations = iterations;
    }
  }

  impl EasingFunction for Cubic
  {
    pub fn apply( &self, t : f32 ) -> f32 
    {
      if t <= 0.0 
      {
        return 0.0;
      }
      else if t >= 1.0 
      {
        return 1.0;
      }

      let mut bezier_t = t;
      for _ in 0..self.iterations
      {
        let slope = 3.0 * ( 1.0 - bezier_t ).powi( 2 ) * self.in_tangent[ 0 ]
        + 6.0 * ( 1.0 - bezier_t ) * bezier_t * self.out_tangent[ 0 ]
        + 3.0 * bezier_t.powi( 2 );
        if slope == 0.0 
        {
          break;
        }
        let x_val = self.get_x( bezier_t ) - t;
        bezier_t -= x_val / slope;
      }

      self.get_y( bezier_t )
    }
  }

  impl_easing_function!( EaseInSine, Cubic, Cubic::new( [ 0.12, 0.0, 0.39, 0.0 ] ) );
  impl_easing_function!( EaseOutSine, Cubic, Cubic::new( [ 0.61, 1.0, 0.88, 1.0 ] ) );
  impl_easing_function!( EaseInOutSine, Cubic, Cubic::new( [ 0.37, 0.0, 0.63, 1.0 ] ) );

  impl_easing_function!( EaseInQuad, Cubic, Cubic::new( [ 0.11, 0.0, 0.5, 0.0 ] ) );
  impl_easing_function!( EaseOutQuad, Cubic, Cubic::new( [ 0.5, 1.0, 0.89, 1.0 ] ) );
  impl_easing_function!( EaseInOutQuad, Cubic, Cubic::new( [ 0.45, 0.0, 0.55, 1.0 ] ) );

  impl_easing_function!( EaseInCubic, Cubic, Cubic::new( [ 0.32, 0.0, 0.67, 0.0 ] ) );
  impl_easing_function!( EaseOutCubic, Cubic, Cubic::new( [ 0.33, 1.0, 0.68, 1.0 ] ) );
  impl_easing_function!( EaseInOutCubic, Cubic, Cubic::new( [ 0.65, 0.0, 0.35, 1.0 ] ) );

  impl_easing_function!( EaseInQuart, Cubic, Cubic::new( [ 0.5, 0.0, 0.75, 0.0 ] ) );
  impl_easing_function!( EaseOutQuart, Cubic, Cubic::new( [ 0.25, 1.0, 0.5, 1.0 ] ) );
  impl_easing_function!( EaseInOutQuart, Cubic, Cubic::new( [ 0.76, 0.0, 0.24, 1.0 ] ) );

  impl_easing_function!( EaseInQuint, Cubic, Cubic::new( [ 0.64, 0.0, 0.78, 0.0 ] ) );
  impl_easing_function!( EaseOutQuint, Cubic, Cubic::new( [ 0.22, 1.0, 0.36, 1.0 ] ) );
  impl_easing_function!( EaseInOutQuint, Cubic, Cubic::new( [ 0.83, 0.0, 0.17, 1.0 ] ) );

  impl_easing_function!( EaseInExpo, Cubic, Cubic::new( [ 0.7, 0.0, 0.84, 0.0 ] ) );
  impl_easing_function!( EaseOutExpo, Cubic, Cubic::new( [ 0.16, 1.0, 0.3, 1.0 ] ) );
  impl_easing_function!( EaseInOutExpo, Cubic, Cubic::new( [ 0.87, 0.0, 0.13, 1.0 ] ) );

  impl_easing_function!( EaseInCirc, Cubic, Cubic::new( [ 0.55, 0.0, 1.0, 0.45 ] ) );
  impl_easing_function!( EaseOutCirc, Cubic, Cubic::new( [ 0.0, 0.55, 0.45, 1.0 ] ) );
  impl_easing_function!( EaseInOutCirc, Cubic, Cubic::new( [ 0.85, 0.0, 0.15, 1.0 ] ) );

  impl_easing_function!( EaseInBack, Cubic, Cubic::new( [ 0.36, 0.0, 0.66, -0.56 ] ) );
  impl_easing_function!( EaseOutBack, Cubic, Cubic::new( [ 0.34, 1.56, 0.64, 1.0 ] ) );
  impl_easing_function!( EaseInOutBack, Cubic, Cubic::new( [ 0.68, -0.6, 0.32, 1.6 ] ) ); 
}

crate::mod_interface!
{
  orphan use
  {
    Cubic,

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
}
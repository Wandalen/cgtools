mod private
{
  use std::marker::PhantomData;

use crate::{impl_easing_function, Animatable};
  use crate::easing::
  {
    base::
    {
      EasingFunction,
      EasingBuilder,
    },
  };

  /// Represents a cubic Bezier curve easing function.
  ///
  /// The curve is defined by two control points: `in_tangent` and `out_tangent`.
  #[ derive( Debug ) ]
  pub struct CubicBezier< A >
  where A : Animatable
  {
    in_tangent : [ f64; 2 ],
    out_tangent : [ f64; 2 ],
    iterations : usize,
    _marker : PhantomData< A >
  }

  impl< A > CubicBezier< A >
  where A : Animatable
  {
    /// Calculates the x-coordinate of the Bezier curve at a given time `t`.
    ///
    /// This is part of the inverse function used to solve for `t`.
    fn get_x( &self, time : f64 ) -> f64
    {
      let one_minus_t = 1.0 - time;
      3.0 * one_minus_t.powi( 2 ) * time * self.in_tangent[ 0 ]
      + 3.0 * one_minus_t * time.powi( 2 ) * self.out_tangent[ 0 ]
      + time.powi( 3 )
    }

    /// Calculates the y-coordinate of the Bezier curve at a given time `t`.
    ///
    /// This represents the final easing value.
    fn get_y( &self, time : f64 ) -> f64
    {
      let one_minus_t = 1.0 - time;
      3.0 * one_minus_t.powi( 2 ) * time * self.in_tangent[ 1 ]
      + 3.0 * one_minus_t * time.powi( 2 ) * self.out_tangent[ 1 ]
      + time.powi( 3 )
    }

    /// Creates a new `CubicBezier` easing function with the specified Bezier curve parameters.
    ///
    /// `parameters` should be an array of four `f64` values representing
    /// `[ in_tangent_x, in_tangent_y, out_tangent_x, out_tangent_y ]`.
    pub fn new( parameters : [ f64; 4 ] ) -> Self
    {
      let [ i1, i2, o1, o2 ] = parameters;
      Self
      {
        in_tangent : [ i1, i2 ],
        out_tangent : [ o1, o2 ],
        iterations : 0,
        _marker : PhantomData
      }
    }

    /// Sets the number of iterations for the easing function's internal calculation.
    ///
    /// Higher values increase precision at the cost of performance.
    pub fn set_iterations( &mut self, iterations : usize )
    {
      self.iterations = iterations;
    }
  }

  impl< A > EasingFunction for CubicBezier< A >
  where A : Animatable
  {
    type AnimatableType = A;

    fn apply( &self, start : Self::AnimatableType, end : Self::AnimatableType, time : f64 ) -> Self::AnimatableType
    {
      if time <= 0.0
      {
        return start;
      }
      if time >= 1.0
      {
        return end;
      }

      let mut bezier_t = time;
      for _ in 0..self.iterations
      {
        let slope = 3.0 * ( 1.0 - bezier_t ).powi( 2 ) * self.in_tangent[ 0 ]
        + 6.0 * ( 1.0 - bezier_t ) * bezier_t * self.out_tangent[ 0 ]
        + 3.0 * bezier_t.powi( 2 );
        if slope == 0.0
        {
          break;
        }
        let x_val = self.get_x( bezier_t ) - time;
        bezier_t -= x_val / slope;
      }

      let time = self.get_y( bezier_t );

      start.interpolate( &end, time )
    }
  }

  impl_easing_function!( EaseInSine, CubicBezier< A >, CubicBezier::< A >::new( [ 0.12, 0.0, 0.39, 0.0 ] ) );
  impl_easing_function!( EaseOutSine, CubicBezier< A >, CubicBezier::< A >::new( [ 0.61, 1.0, 0.88, 1.0 ] ) );
  impl_easing_function!( EaseInOutSine, CubicBezier< A >, CubicBezier::< A >::new( [ 0.37, 0.0, 0.63, 1.0 ] ) );

  impl_easing_function!( EaseInQuad, CubicBezier< A >, CubicBezier::< A >::new( [ 0.11, 0.0, 0.5, 0.0 ] ) );
  impl_easing_function!( EaseOutQuad, CubicBezier< A >, CubicBezier::< A >::new( [ 0.5, 1.0, 0.89, 1.0 ] ) );
  impl_easing_function!( EaseInOutQuad, CubicBezier< A >, CubicBezier::< A >::new( [ 0.45, 0.0, 0.55, 1.0 ] ) );

  impl_easing_function!( EaseInCubic, CubicBezier< A >, CubicBezier::< A >::new( [ 0.32, 0.0, 0.67, 0.0 ] ) );
  impl_easing_function!( EaseOutCubic, CubicBezier< A >, CubicBezier::< A >::new( [ 0.33, 1.0, 0.68, 1.0 ] ) );
  impl_easing_function!( EaseInOutCubic, CubicBezier< A >, CubicBezier::< A >::new( [ 0.65, 0.0, 0.35, 1.0 ] ) );

  impl_easing_function!( EaseInQuart, CubicBezier< A >, CubicBezier::< A >::new( [ 0.5, 0.0, 0.75, 0.0 ] ) );
  impl_easing_function!( EaseOutQuart, CubicBezier< A >, CubicBezier::< A >::new( [ 0.25, 1.0, 0.5, 1.0 ] ) );
  impl_easing_function!( EaseInOutQuart, CubicBezier< A >, CubicBezier::< A >::new( [ 0.76, 0.0, 0.24, 1.0 ] ) );

  impl_easing_function!( EaseInQuint, CubicBezier< A >, CubicBezier::< A >::new( [ 0.64, 0.0, 0.78, 0.0 ] ) );
  impl_easing_function!( EaseOutQuint, CubicBezier< A >, CubicBezier::< A >::new( [ 0.22, 1.0, 0.36, 1.0 ] ) );
  impl_easing_function!( EaseInOutQuint, CubicBezier< A >, CubicBezier::< A >::new( [ 0.83, 0.0, 0.17, 1.0 ] ) );

  impl_easing_function!( EaseInExpo, CubicBezier< A >, CubicBezier::< A >::new( [ 0.7, 0.0, 0.84, 0.0 ] ) );
  impl_easing_function!( EaseOutExpo, CubicBezier< A >, CubicBezier::< A >::new( [ 0.16, 1.0, 0.3, 1.0 ] ) );
  impl_easing_function!( EaseInOutExpo, CubicBezier< A >, CubicBezier::< A >::new( [ 0.87, 0.0, 0.13, 1.0 ] ) );

  impl_easing_function!( EaseInCirc, CubicBezier< A >, CubicBezier::< A >::new( [ 0.55, 0.0, 1.0, 0.45 ] ) );
  impl_easing_function!( EaseOutCirc, CubicBezier< A >, CubicBezier::< A >::new( [ 0.0, 0.55, 0.45, 1.0 ] ) );
  impl_easing_function!( EaseInOutCirc, CubicBezier< A >, CubicBezier::< A >::new( [ 0.85, 0.0, 0.15, 1.0 ] ) );

  impl_easing_function!( EaseInBack, CubicBezier< A >, CubicBezier::< A >::new( [ 0.36, 0.0, 0.66, -0.56 ] ) );
  impl_easing_function!( EaseOutBack, CubicBezier< A >, CubicBezier::< A >::new( [ 0.34, 1.56, 0.64, 1.0 ] ) );
  impl_easing_function!( EaseInOutBack, CubicBezier< A >, CubicBezier::< A >::new( [ 0.68, -0.6, 0.32, 1.6 ] ) );
}

crate::mod_interface!
{
  orphan use
  {
    CubicBezier,

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

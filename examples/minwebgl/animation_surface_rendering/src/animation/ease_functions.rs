#[ allow( unused ) ]
mod private
{
  use crate::model::animated;
  use interpoli::Value;

  /// Defines an easing function with two optional `(x, y)` Bezier control points.
  type EaseFunction = [ Option< [ f64; 2 ] > ; 2 ];

  pub const EASE_IN_SINE : EaseFunction = [ Some( [ 0.12, 0.0] ), Some( [ 0.39, 0.0 ] ) ];
  pub const EASE_OUT_SINE : EaseFunction = [ Some( [ 0.61, 1.0 ] ), Some( [ 0.88, 1.0 ] ) ];
  pub const EASE_IN_OUT_SINE : EaseFunction = [ Some( [ 0.37, 0.0 ] ), Some( [ 0.63, 1.0 ] ) ];

  pub const EASE_IN_QUAD : EaseFunction = [ Some( [ 0.11, 0.0 ] ), Some( [ 0.5, 0.0 ] ) ];
  pub const EASE_OUT_QUAD : EaseFunction = [ Some( [ 0.5, 1.0 ] ), Some( [ 0.89, 1.0 ] ) ];
  pub const EASE_IN_OUT_QUAD : EaseFunction = [ Some( [ 0.45, 0.0 ] ), Some( [ 0.55, 1.0 ] ) ];

  pub const EASE_IN_CUBIC : EaseFunction = [ Some( [ 0.32, 0.0 ] ), Some( [ 0.67, 0.0 ] ) ];
  pub const EASE_OUT_CUBIC : EaseFunction = [ Some( [ 0.33, 1.0 ] ), Some( [ 0.68, 1.0 ] ) ];
  pub const EASE_IN_OUT_CUBIC : EaseFunction = [ Some( [ 0.65, 0.0 ] ), Some( [ 0.35, 1.0 ] ) ];

  pub const EASE_IN_QUART : EaseFunction = [ Some( [ 0.5, 0.0 ] ), Some( [ 0.75, 0.0 ] ) ];
  pub const EASE_OUT_QUART : EaseFunction = [ Some( [ 0.25, 1.0 ] ), Some( [ 0.5, 1.0 ] ) ];
  pub const EASE_IN_OUT_QUART : EaseFunction = [ Some( [ 0.76, 0.0 ] ), Some( [ 0.24, 1.0 ] ) ];

  pub const EASE_IN_QUINT : EaseFunction = [ Some( [ 0.64, 0.0 ] ), Some( [ 0.78, 0.0 ] ) ];
  pub const EASE_OUT_QUINT : EaseFunction = [ Some( [ 0.22, 1.0 ] ), Some( [ 0.36, 1.0 ] ) ];
  pub const EASE_IN_OUT_QUINT : EaseFunction = [ Some( [ 0.83, 0.0 ] ), Some( [ 0.17, 1.0 ] ) ];

  pub const EASE_IN_EXPO : EaseFunction = [ Some( [ 0.7, 0.0 ] ), Some( [ 0.84, 0.0 ] ) ];
  pub const EASE_OUT_EXPO : EaseFunction = [ Some( [ 0.16, 1.0 ] ), Some( [ 0.3, 1.0 ] ) ];
  pub const EASE_IN_OUT_EXPO : EaseFunction = [ Some( [ 0.87, 0.0 ] ), Some( [ 0.13, 1.0 ] ) ];

  pub const EASE_IN_CIRC : EaseFunction = [ Some( [ 0.55, 0.0 ] ), Some( [ 1.0, 0.45 ] ) ];
  pub const EASE_OUT_CIRC : EaseFunction = [ Some( [ 0.0, 0.55 ] ), Some( [ 0.45, 1.0 ] ) ];
  pub const EASE_IN_OUT_CIRC : EaseFunction = [ Some( [ 0.85, 0.0 ] ), Some( [ 0.15, 1.0 ] ) ];

  pub const EASE_IN_BACK : EaseFunction = [ Some( [ 0.36, 0.0 ] ), Some( [ 0.66, -0.56 ] ) ];
  pub const EASE_OUT_BACK : EaseFunction = [ Some( [ 0.34, 1.56 ] ), Some( [ 0.64, 1.0 ] ) ];
  pub const EASE_IN_OUT_BACK : EaseFunction = [ Some( [ 0.68, -0.6 ] ), Some( [ 0.32, 1.6 ] ) ];

  /// A linear easing function, which provides a constant rate of change.
  pub const LINEAR : EaseFunction = [ None, None ];

  /// Creates a simple two-keyframe animation with a specified easing function.
  ///
  /// # Arguments
  ///
  /// * `( f1, f2 )` - A tuple containing the start and end frames of the animation.
  /// * `( v1, v2 )` - A tuple containing the start and end values of the animation.
  /// * `ease_function` - The easing function to apply between the two keyframes.
  ///
  /// # Returns
  ///
  /// An animated `Value` that transitions from `v1` to `v2` over the specified frame range.
  pub fn ease< T : interpoli::Tween >
  ( 
    ( f1, f2 ) : ( f64, f64 ),
    ( v1, v2 ) : ( T, T ),
    ease_function : EaseFunction
  ) -> Value< T >
  {
    animated
    (
      vec!
      [
        crate::animation::model::Keyframe::new( f1, ease_function, false, v1 ),
        crate::animation::model::Keyframe::new( f2, ease_function, false, v2 ),
      ]
    ) 
  }
}

crate::mod_interface!
{
  orphan use
  {
    EASE_IN_SINE,
    EASE_OUT_SINE,
    EASE_IN_OUT_SINE,

    EASE_IN_QUAD,
    EASE_OUT_QUAD,
    EASE_IN_OUT_QUAD,
    
    EASE_IN_CUBIC,
    EASE_OUT_CUBIC,
    EASE_IN_OUT_CUBIC,
    
    EASE_IN_QUART,
    EASE_OUT_QUART,
    EASE_IN_OUT_QUART,
    
    EASE_IN_QUINT,
    EASE_OUT_QUINT,
    EASE_IN_OUT_QUINT,
    
    EASE_IN_CIRC,
    EASE_OUT_CIRC,
    EASE_IN_OUT_CIRC,

    EASE_IN_BACK,
    EASE_OUT_BACK,
    EASE_IN_OUT_BACK,

    LINEAR,

    ease
  };
}
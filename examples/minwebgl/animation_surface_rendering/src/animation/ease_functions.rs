mod private
{
  use crate::model::animated;
  use interpoli::Value;

  pub const EASE_IN_SINE : [f64; 4] = [ 0.12, 0.0, 0.39, 0.0 ];
  pub const EASE_OUT_SINE : [f64; 4] = [ 0.61, 1.0, 0.88, 1.0 ];
  pub const EASE_IN_OUT_SINE : [f64; 4] = [ 0.37, 0.0, 0.63, 1.0 ];
  
  pub const EASE_IN_QUAD : [f64; 4] = [ 0.11, 0.0, 0.5, 0.0 ];
  pub const EASE_OUT_QUAD : [f64; 4] = [ 0.5, 1.0, 0.89, 1.0 ];
  pub const EASE_IN_OUT_QUAD : [f64; 4] = [ 0.45, 0.0, 0.55, 1.0 ];
  
  pub const EASE_IN_CUBIC : [f64; 4] = [ 0.32, 0.0, 0.67, 0.0 ];
  pub const EASE_OUT_CUBIC : [f64; 4] = [ 0.33, 1.0, 0.68, 1.0 ];
  pub const EASE_IN_OUT_CUBIC : [f64; 4] = [ 0.65, 0.0, 0.35, 1.0 ];
  
  pub const EASE_IN_QUART : [f64; 4] = [ 0.5, 0.0, 0.75, 0.0 ];
  pub const EASE_OUT_QUART : [f64; 4] = [ 0.25, 1.0, 0.5, 1.0 ];
  pub const EASE_IN_OUT_QUART : [f64; 4] = [ 0.76, 0.0, 0.24, 1.0 ];
  
  pub const EASE_IN_QUINT : [f64; 4] = [ 0.64, 0.0, 0.78, 0.0 ];
  pub const EASE_OUT_QUINT : [f64; 4] = [ 0.22, 1.0, 0.36, 1.0 ];
  pub const EASE_IN_OUT_QUINT : [f64; 4] = [ 0.83, 0.0, 0.17, 1.0 ];
  
  pub const EASE_IN_EXPO : [f64; 4] = [ 0.7, 0.0, 0.84, 0.0 ];
  pub const EASE_OUT_EXPO : [f64; 4] = [ 0.16, 1.0, 0.3, 1.0 ];
  pub const EASE_IN_OUT_EXPO : [f64; 4] = [ 0.87, 0.0, 0.13, 1.0 ];

  pub const EASE_IN_CIRC : [f64; 4] = [ 0.55, 0.0, 1.0, 0.45 ];
  pub const EASE_OUT_CIRC : [f64; 4] = [ 0.0, 0.55, 0.45, 1.0 ];
  pub const EASE_IN_OUT_CIRC : [f64; 4] = [ 0.85, 0.0, 0.15, 1.0 ];

  pub const EASE_IN_BACK : [f64; 4] = [ 0.36, 0.0, 0.66, -0.56 ];
  pub const EASE_OUT_BACK : [f64; 4] = [ 0.34, 1.56, 0.64, 1.0 ];
  pub const EASE_IN_OUT_BACK : [f64; 4] = [ 0.68, -0.6, 0.32, 1.6 ];

  pub fn ease< T : interpoli::Tween >
  ( 
    ( f1, f2 ) : ( f64, f64 ),
    ( v1, v2 ) : ( T, T ),
    [ x1, y1, x2, y2 ] : [ f64; 4 ] 
  ) -> Value< T >
  {
    let bezier_parameters = [ Some( [ x1, y1 ] ), Some( [ x2, y2 ] ) ];
    animated
    (
      vec!
      [
        ( f1, bezier_parameters, false, v1 ),
        ( f2, bezier_parameters, false, v2 ),
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

    ease
  };
}
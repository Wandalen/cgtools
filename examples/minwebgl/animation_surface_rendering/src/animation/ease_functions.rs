mod private
{
  use crate::model::animated;
  use interpoli::Value;

  pub fn ease_in_out_circ< T : interpoli::Tween >
  ( 
    ( f1, f2 ) : ( f64, f64 ),
    ( v1, v2 ) : ( T, T ) 
  ) -> Value< T >
  {
    animated
    (
      vec!
      [
        ( f1, [ Some( [ 0.85, 0.0 ] ), Some( [ 0.15, 1.0 ] ) ], false, v1 ),
        ( f2, [ Some( [ 0.85, 0.0 ] ), Some( [ 0.15, 1.0 ] ) ], false, v2 ),
      ]
    ) 
  }
}

crate::mod_interface!
{
  orphan use
  {
    ease_in_out_circ
  };
}
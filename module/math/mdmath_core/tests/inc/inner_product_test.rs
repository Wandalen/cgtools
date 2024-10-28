use super::*;

#[ test ]
fn test_dot_product()
{
  use the_module::vector;

  // Test with typical vectors
  let vec_a = [ 1.0, 2.0, 3.0 ];
  let vec_b = [ 4.0, 5.0, 6.0 ];
  let result = vector::dot( &vec_a, &vec_b );
  assert_eq!( result, 32.0, "Dot product calculation failed for typical vectors" );

  // Test with negative numbers
  let vec_c = [ -1.0, -2.0, -3.0 ];
  let vec_d = [ 4.0, 5.0, 6.0 ];
  let result_neg = vector::dot( &vec_c, &vec_d );
  assert_eq!( result_neg, -32.0, "Dot product calculation failed for negative numbers" );

  // Test with zero vectors
  let vec_zero = [ 0.0, 0.0, 0.0 ];
  let got = vector::dot( &vec_a, &vec_zero );
  assert_eq!( got, 0.0, "Dot product calculation failed for zero vector" );

  // Test with empty vectors (edge case)
  let vec_empty : [ f32; 0 ] = [];
  let result_empty = vector::dot( &vec_empty, &vec_empty );
  assert_eq!( result_empty, 0.0, "Dot product calculation failed for empty vectors" );

}

#[ test ]
fn test_magnitude2()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
  };

  let vec_a = [ 1.0, 2.0, 3.0 ];
  let result = vector::mag2( &vec_a );
  assert_ulps_eq!( result, 14.0 );

  let vec_zero = [ 0.0, 0.0, 0.0 ];
  let got = vector::mag2( &vec_zero );
  assert_ulps_eq!( got, 0.0 );

  let vec_empty : [ f32; 0 ] = [];
  let result_empty = vector::mag2( &vec_empty );
  assert_ulps_eq!( result_empty, 0.0 );
}

#[ test ]
fn test_magnitude()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
  };

  let vec_a = [ 3.0, 4.0 ];
  let result = vector::mag( &vec_a );
  assert_ulps_eq!( result, 5.0 );

  let vec_zero = [ 0.0, 0.0 ];
  let got = vector::mag( &vec_zero );
  assert_ulps_eq!( got, 0.0 );

  let vec_empty : [ f32; 0 ] = [];
  let result_empty = vector::mag( &vec_empty );
  assert_ulps_eq!( result_empty, 0.0 );
}

#[ test ]
fn test_normalize()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    Float,
    vector::{ IterFloat, IterExt },
  };

  // Test with a typical vector
  let vec_a = [ 3.0, 4.0 ];
  let mut result = vec_a.clone();
  vector::normalize( &mut result, &vec_a );
  let expected = [ 0.6, 0.8 ];
  for ( a, b ) in result.iter().zip( expected.iter() )
  {
    assert_ulps_eq!( a, b );
  }

  // Test with a zero vector
  let vec_zero = [ 0.0, 0.0 ];
  let mut got = vec_zero.clone();
  vector::normalize( &mut got, &vec_zero );
  assert!( got.iter().is_nan().all_true(), "Expected NaN, got {:?}", got );

  for value in got.iter()
  {
    assert!( value.is_nan(), "Expected NaN, got {}", value );
  }

}

#[ test ]
fn test_normalized()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    Float,
  };

  let vec_a = [ 3.0, 4.0 ];
  let result = vector::normalized( &vec_a );
  let expected = [ 0.6, 0.8 ];
  for ( a, b ) in result.iter().zip( expected.iter() )
  {
    assert_ulps_eq!( a, b );
  }

  let vec_zero = [ 0.0, 0.0 ];
  let got = vector::normalized( &vec_zero );

  for value in got.iter()
  {
    assert!( value.is_nan(), "Expected NaN, got {}", value );
  }

}

#[ test ]
fn test_normalize_to()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    Float,
  };

  let mut vec_a = [ 3.0, 4.0 ];
  vector::normalize_to( &mut vec_a, 10.0 );
  let expected = [ 6.0, 8.0 ];
  for ( a, b ) in vec_a.iter().zip( expected.iter() )
  {
    assert_ulps_eq!( a, b );
  }

  let mut got = [ 0.0, 0.0 ];
  vector::normalize_to( &mut got, 10.0 );

  for value in got.iter()
  {
    assert!( value.is_nan(), "Expected NaN, got {}", value );
  }

}

#[ test ]
fn test_normalized_to()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    Float,
  };

  let vec_a = [ 3.0, 4.0 ];
  let result = vector::normalized_to( &vec_a, 10.0 );
  let expected = [ 6.0, 8.0 ];
  for ( a, b ) in result.iter().zip( expected.iter() )
  {
    assert_ulps_eq!( a, b );
  }

  let vec_zero = [ 0.0, 0.0 ];
  let got = vector::normalized_to( &vec_zero, 10.0 );
  for value in got.iter()
  {
    assert!( value.is_nan(), "Expected NaN, got {}", value );
  }
}

#[ test ]
fn test_project_on()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    // Float,
  };

  let mut vec_a = [ 1.0, 2.0, 3.0 ];
  let vec_b = [ 4.0, 5.0, 6.0 ];
  vector::project_on( &mut vec_a, &vec_b );
  let expected = [ 1.6623376623376624, 2.077922077922078, 2.4935064935064934 ];
  for ( a, b ) in vec_a.iter().zip( expected.iter() )
  {
    assert_ulps_eq!( a, b );
    // qqq : xxx : make that working : assert_ulps_eq!( vec_a, expected );
  }

  let mut vec_zero = [ 0.0, 0.0, 0.0 ];
  vector::project_on( &mut vec_zero, &vec_b );
  assert_eq!( vec_zero, [ 0.0, 0.0, 0.0 ], "Projection failed for zero vector" );
}

#[ test ]
fn test_projected_on()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    // Float,
  };

  let vec_a = [ 1.0, 2.0, 3.0 ];
  let vec_b = [ 4.0, 5.0, 6.0 ];
  let result = vector::projected_on( &vec_a, &vec_b );
  let expected = [ 1.6623376623376624, 2.077922077922078, 2.4935064935064934 ];
  // xxx : rid of cylce here
  for ( a, b ) in result.iter().zip( expected.iter() )
  {
    assert_ulps_eq!( a, b, max_ulps = 10000 );
  }

  let vec_zero = [ 0.0, 0.0, 0.0 ];
  let got = vector::projected_on( &vec_zero, &vec_b );
  assert_eq!( got, [ 0.0, 0.0, 0.0 ], "Projected on function failed for zero vector" );
}

#[ test ]
fn test_angle()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    // Float,
  };

  let vec_a = [ 1.0, 0.0 ];
  let vec_b = [ 0.0, 1.0 ];
  let result = vector::angle( &vec_a, &vec_b );
  assert_ulps_eq!( result, std::f32::consts::FRAC_PI_2 );

  let vec_zero = [ 0.0, 0.0 ];
  let got = vector::angle( &vec_a, &vec_zero );
  assert!( got.is_nan(), "Angle calculation failed for zero vector" );
}

#[ test ]
fn test_is_orthogonal()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    // Float,
  };

  // Test with orthogonal vectors
  let vec_a = [ 1.0, 0.0 ];
  let vec_b = [ 0.0, 1.0 ];
  assert!( vector::is_orthogonal( &vec_a, &vec_b ), "Orthogonal test failed for orthogonal vectors" );

  // Test with non-orthogonal vectors
  let vec_c = [ 1.0, 1.0 ];
  let vec_d = [ 1.0, 0.0 ];
  assert!( !vector::is_orthogonal( &vec_c, &vec_d ), "Orthogonal test failed for non-orthogonal vectors" );

  // Test with zero vector
  let vec_zero = [ 0.0, 0.0 ];
  assert!( vector::is_orthogonal( &vec_a, &vec_zero ), "Orthogonal test failed for zero vector" );
}

#[ test ]
fn test_cross_mut()
{
  use the_module::
  {
    vector,
    assert_ulps_eq
  };

  let mut vec_a = [ 1.0, 0.0, 0.0 ];
  let vec_b = [ 0.0, 1.0, 0.0 ];
  vector::cross_mut( &mut vec_a, &vec_b );

  let exp = [ 0.0, 0.0, 1.0 ];
  for ( r, e ) in vec_a.iter().zip( exp.iter() )
  {
    assert_ulps_eq!( r, e );
  }

  let mut vec_a = [ 1.0, 2.0, 3.0 ];
  let vec_b = [ 1.0, 5.0, 7.0 ];
  vector::cross_mut( &mut vec_a, &vec_b );

  let exp = [ -1.0, -4.0, 3.0 ];
  for ( r, e ) in vec_a.iter().zip( exp.iter() )
  {
    assert_ulps_eq!( r, e );
  }
}

#[ test ]
fn test_cross()
{
  use the_module::
  {
    vector,
    assert_ulps_eq
  };

  let vec_a = [ 1.0, 0.0, 0.0 ];
  let vec_b = [ 0.0, 1.0, 0.0 ];
  let res = vector::cross( &vec_a, &vec_b );

  let exp = [ 0.0, 0.0, 1.0 ];
  for ( r, e ) in res.iter().zip( exp.iter() )
  {
    assert_ulps_eq!( r, e );
  }

  let vec_a = [ 1.0, 2.0, 3.0 ];
  let vec_b = [ 1.0, 5.0, 7.0 ];
  let res = vector::cross( &vec_a, &vec_b );

  let exp = [ -1.0, -4.0, 3.0 ];
  for ( r, e ) in res.iter().zip( exp.iter() )
  {
    assert_ulps_eq!( r, e );
  }
}
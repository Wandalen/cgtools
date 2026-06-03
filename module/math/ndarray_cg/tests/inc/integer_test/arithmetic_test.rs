//! Integer vector and matrix arithmetic — add / sub / mul / div / rem, scalar
//! ops, dot, cross, mat×mat, mat×vec. Parameterized over element type via macros so
//! every integer primitive (`i32`, `i64`, `u32`, `u64`) gets covered.
//!
//! `cross` and `distance_squared` require a signed element type because the
//! underlying subtraction can produce negative intermediate values; they are
//! tested separately for `i32` and `i64`.

use super::*;

fn vector_add_sub_generic< E >()
where
  E : the_module::MatNum + From< u8 > + PartialEq + core::fmt::Debug,
{
  use the_module::Vector;
  let a = Vector::< E, 3 >::from_array( [ E::from( 1 ), E::from( 2 ), E::from( 3 ) ] );
  let b = Vector::< E, 3 >::from_array( [ E::from( 4 ), E::from( 5 ), E::from( 6 ) ] );
  let sum = a + b;
  assert_eq!( sum, Vector::< E, 3 >::from_array( [ E::from( 5 ), E::from( 7 ), E::from( 9 ) ] ) );
  let diff = b - a;
  assert_eq!( diff, Vector::< E, 3 >::from_array( [ E::from( 3 ), E::from( 3 ), E::from( 3 ) ] ) );
}

fn vector_scalar_mul_div_generic< E >()
where
  E : the_module::MatNum + From< u8 > + PartialEq + core::fmt::Debug,
{
  use the_module::Vector;
  let v = Vector::< E, 3 >::from_array( [ E::from( 2 ), E::from( 4 ), E::from( 6 ) ] );
  let scaled = v * E::from( 3 );
  assert_eq!( scaled, Vector::< E, 3 >::from_array( [ E::from( 6 ), E::from( 12 ), E::from( 18 ) ] ) );
  let halved = scaled / E::from( 3 );
  assert_eq!( halved, v );
}

fn vector_rem_generic< E >()
where
  E : the_module::MatNum + From< u8 > + PartialEq + core::fmt::Debug,
{
  use the_module::Vector;
  let a = Vector::< E, 3 >::from_array( [ E::from( 7 ), E::from( 10 ), E::from( 15 ) ] );
  let b = Vector::< E, 3 >::from_array( [ E::from( 3 ), E::from( 4 ), E::from( 6 ) ] );

  // vector % vector
  assert_eq!( a % b, Vector::< E, 3 >::from_array( [ E::from( 1 ), E::from( 2 ), E::from( 3 ) ] ) );
  // vector % scalar
  assert_eq!( a % E::from( 3 ), Vector::< E, 3 >::from_array( [ E::from( 1 ), E::from( 1 ), E::from( 0 ) ] ) );

  // &vector % &vector  (reference form)
  assert_eq!( &a % &b, Vector::< E, 3 >::from_array( [ E::from( 1 ), E::from( 2 ), E::from( 3 ) ] ) );
  // &vector % scalar  (reference form)
  assert_eq!( &a % E::from( 3 ), Vector::< E, 3 >::from_array( [ E::from( 1 ), E::from( 1 ), E::from( 0 ) ] ) );

  // %= vector
  let mut c = a;
  c %= b;
  assert_eq!( c, Vector::< E, 3 >::from_array( [ E::from( 1 ), E::from( 2 ), E::from( 3 ) ] ) );
  // %= scalar
  let mut d = a;
  d %= E::from( 3 );
  assert_eq!( d, Vector::< E, 3 >::from_array( [ E::from( 1 ), E::from( 1 ), E::from( 0 ) ] ) );
}

fn vector_div_signed_generic< E >()
where
  E : the_module::MatNum + ::num_traits::Signed + From< u8 > + PartialEq + core::fmt::Debug,
{
  use the_module::Vector;
  // Integer division truncates toward zero, so negative dividends round up:
  // `-7 / 3 == -2`, not `-3`.
  let a = Vector::< E, 3 >::from_array( [ -E::from( 7 ), -E::from( 10 ), -E::from( 15 ) ] );
  let b = Vector::< E, 3 >::from_array( [ E::from( 3 ), E::from( 4 ), E::from( 6 ) ] );

  // vector / vector
  assert_eq!( a / b, Vector::< E, 3 >::from_array( [ -E::from( 2 ), -E::from( 2 ), -E::from( 2 ) ] ) );
  // vector / scalar
  assert_eq!( a / E::from( 3 ), Vector::< E, 3 >::from_array( [ -E::from( 2 ), -E::from( 3 ), -E::from( 5 ) ] ) );

  // /= vector
  let mut c = a;
  c /= b;
  assert_eq!( c, Vector::< E, 3 >::from_array( [ -E::from( 2 ), -E::from( 2 ), -E::from( 2 ) ] ) );
  // /= scalar
  let mut d = a;
  d /= E::from( 3 );
  assert_eq!( d, Vector::< E, 3 >::from_array( [ -E::from( 2 ), -E::from( 3 ), -E::from( 5 ) ] ) );
}

fn vector_dot_generic< E >()
where
  E : the_module::MatNum + From< u8 > + PartialEq + core::fmt::Debug,
{
  use the_module::Vector;
  let a = Vector::< E, 3 >::from_array( [ E::from( 1 ), E::from( 2 ), E::from( 3 ) ] );
  let b = Vector::< E, 3 >::from_array( [ E::from( 4 ), E::from( 5 ), E::from( 6 ) ] );
  assert_eq!( a.dot( &b ), E::from( 32 ) );
  assert_eq!( a.mag2(), E::from( 14 ) );
}

fn vector_distance_squared_signed_generic< E >()
where
  E : the_module::MatNum + ::num_traits::Signed + From< u8 > + PartialEq + core::fmt::Debug,
{
  use the_module::Vector;
  let a = Vector::< E, 3 >::from_array( [ E::from( 1 ), E::from( 2 ), E::from( 3 ) ] );
  let b = Vector::< E, 3 >::from_array( [ E::from( 4 ), E::from( 5 ), E::from( 6 ) ] );
  assert_eq!( a.distance_squared( &b ), E::from( 27 ) );
  assert_eq!( b.distance_squared( &a ), E::from( 27 ) );
}

fn vec3_cross_signed_generic< E >()
where
  E : the_module::MatNum + ::num_traits::Signed + From< u8 > + PartialEq + core::fmt::Debug,
{
  use the_module::Vector;
  let x = Vector::< E, 3 >::new( E::from( 1 ), E::from( 0 ), E::from( 0 ) );
  let y = Vector::< E, 3 >::new( E::from( 0 ), E::from( 1 ), E::from( 0 ) );
  let z = x.cross( y );
  assert_eq!( z, Vector::< E, 3 >::new( E::from( 0 ), E::from( 0 ), E::from( 1 ) ) );
  // Anti-commutativity: y × x = -z (negative z component, requires signed type)
  let neg_z = y.cross( x );
  let minus_one = E::zero() - E::from( 1 );
  assert_eq!( neg_z, Vector::< E, 3 >::new( E::from( 0 ), E::from( 0 ), minus_one ) );
}

fn vector_neg_signed_generic< E >()
where
  E : the_module::MatNum + ::num_traits::Signed + ::num_traits::Bounded + From< u8 > + PartialEq + core::fmt::Debug,
{
  use the_module::Vector;
  let ( one, two, three ) = ( E::from( 1 ), E::from( 2 ), E::from( 3 ) );
  let v = Vector::< E, 3 >::from_array( [ one, -two, three ] );
  let expected = Vector::< E, 3 >::from_array( [ -one, two, -three ] );

  // owned Neg
  assert_eq!( -v, expected );
  // reference Neg
  assert_eq!( -&v, expected );

  // Boundary: `E::MIN` itself overflows on negation (documented panic), but
  // `E::MIN + 1` is the safe edge and negates to `E::MAX`. The `E::MIN` case is
  // intentionally not exercised here.
  let near_min = Vector::< E, 3 >::from_array( [ E::min_value() + one, three, -three ] );
  let near_min_neg = Vector::< E, 3 >::from_array( [ E::max_value(), -three, three ] );
  assert_eq!( -near_min, near_min_neg );
}

fn vector_rem_signed_generic< E >()
where
  E : the_module::MatNum + ::num_traits::Signed + From< u8 > + PartialEq + core::fmt::Debug,
{
  use the_module::Vector;
  // Rust's `%` keeps the sign of the dividend: `-7 % 3 == -1`, not `2`.
  let a = Vector::< E, 3 >::from_array( [ -E::from( 7 ), -E::from( 10 ), -E::from( 15 ) ] );
  let b = Vector::< E, 3 >::from_array( [ E::from( 3 ), E::from( 4 ), E::from( 6 ) ] );

  // vector % vector
  assert_eq!( a % b, Vector::< E, 3 >::from_array( [ -E::from( 1 ), -E::from( 2 ), -E::from( 3 ) ] ) );
  // vector % scalar
  assert_eq!( a % E::from( 3 ), Vector::< E, 3 >::from_array( [ -E::from( 1 ), -E::from( 1 ), E::from( 0 ) ] ) );

  // %= vector
  let mut c = a;
  c %= b;
  assert_eq!( c, Vector::< E, 3 >::from_array( [ -E::from( 1 ), -E::from( 2 ), -E::from( 3 ) ] ) );
  // %= scalar
  let mut d = a;
  d %= E::from( 3 );
  assert_eq!( d, Vector::< E, 3 >::from_array( [ -E::from( 1 ), -E::from( 1 ), E::from( 0 ) ] ) );
}

fn mat_add_sub_generic< E, D >()
where
  E : the_module::MatNum + From< u8 > + PartialEq + core::fmt::Debug,
  D : the_module::mat::Descriptor + Copy,
  the_module::Mat< 2, 2, E, D > : the_module::IndexingMut + the_module::RawSliceMut< Scalar = E > + the_module::Indexable< Index = the_module::Ix2 >,
{
  use the_module::Mat;
  let a = Mat::< 2, 2, E, D >::default().set_raw
  ([
    E::from( 1 ), E::from( 2 ),
    E::from( 3 ), E::from( 4 ),
  ]);
  let b = Mat::< 2, 2, E, D >::default().set_raw
  ([
    E::from( 5 ), E::from( 6 ),
    E::from( 7 ), E::from( 8 ),
  ]);
  let sum = a + b;
  let exp = Mat::< 2, 2, E, D >::default().set_raw
  ([
    E::from( 6 ),  E::from( 8 ),
    E::from( 10 ), E::from( 12 ),
  ]);
  assert_eq!( sum.raw_slice(), exp.raw_slice() );
}

fn mat_mat_mul_generic< E, D >()
where
  E : the_module::MatNum + From< u8 > + PartialEq + core::fmt::Debug,
  D : the_module::mat::Descriptor + Copy,
  the_module::Mat< 2, 2, E, D > :
    the_module::IndexingMut +
    the_module::RawSliceMut< Scalar = E > +
    the_module::Indexable< Index = the_module::Ix2 > +
    the_module::ScalarMut< Scalar = E, Index = the_module::Ix2 >,
{
  use the_module::Mat;
  // Identity * A = A
  let a = Mat::< 2, 2, E, D >::default().set_raw
  ([
    E::from( 2 ), E::from( 3 ),
    E::from( 5 ), E::from( 7 ),
  ]);
  let i = Mat::< 2, 2, E, D >::identity();
  let r = i * a;
  assert_eq!( r.raw_slice(), a.raw_slice() );

  // Non-identity * non-identity, hand-computed, to exercise the inner-product
  // accumulation (an off-by-one or transposed index survives the identity case
  // above but fails here). Built via `from_row_major` so the expectation holds
  // for both row- and column-major descriptors.
  // [ [ 2, 3 ], [ 5, 7 ] ] * [ [ 1, 2 ], [ 3, 4 ] ] = [ [ 11, 16 ], [ 26, 38 ] ]
  //   (1,1) = 5*2 + 7*4 = 38
  let lhs = Mat::< 2, 2, E, D >::from_row_major
  ([
    E::from( 2 ), E::from( 3 ),
    E::from( 5 ), E::from( 7 ),
  ]);
  let rhs = Mat::< 2, 2, E, D >::from_row_major
  ([
    E::from( 1 ), E::from( 2 ),
    E::from( 3 ), E::from( 4 ),
  ]);
  let exp = Mat::< 2, 2, E, D >::from_row_major
  ([
    E::from( 11 ), E::from( 16 ),
    E::from( 26 ), E::from( 38 ),
  ]);
  assert_eq!( ( lhs * rhs ).raw_slice(), exp.raw_slice() );
}

fn mat_vec_mul_generic< E, D >()
where
  E : the_module::MatNum + From< u8 > + PartialEq + core::fmt::Debug,
  D : the_module::mat::Descriptor + Copy,
  the_module::Mat< 3, 3, E, D > :
    the_module::IndexingMut +
    the_module::RawSliceMut< Scalar = E > +
    the_module::Indexable< Index = the_module::Ix2 > +
    the_module::ScalarMut< Scalar = E, Index = the_module::Ix2 > +
    the_module::IndexingRef< Scalar = E >,
{
  use the_module::{ Mat, Vector };
  let v = Vector::< E, 3 >::from_array( [ E::from( 1 ), E::from( 2 ), E::from( 3 ) ] );

  let m = Mat::< 3, 3, E, D >::identity();
  assert_eq!( m * v, v );

  // Non-identity, asymmetric matrix so a row/column transpose in the product is
  // caught (identity * v = v cannot distinguish the two). `from_row_major` keeps
  // the expectation layout-independent.
  // [ [ 1, 2, 3 ], [ 4, 5, 6 ], [ 7, 8, 9 ] ] * [ 1, 2, 3 ]^T = [ 14, 32, 50 ]
  //   row 1 = 4*1 + 5*2 + 6*3 = 32
  let n = Mat::< 3, 3, E, D >::from_row_major
  ([
    E::from( 1 ), E::from( 2 ), E::from( 3 ),
    E::from( 4 ), E::from( 5 ), E::from( 6 ),
    E::from( 7 ), E::from( 8 ), E::from( 9 ),
  ]);
  let exp = Vector::< E, 3 >::from_array( [ E::from( 14 ), E::from( 32 ), E::from( 50 ) ] );
  assert_eq!( n * v, exp );
}

macro_rules! integer_arithmetic_tests
{
  ( $( $ty:ident ),* $(,)? ) =>
  {
    $(
      mod $ty
      {
        use super::*;

        #[ test ]
        fn vector_add_sub() { vector_add_sub_generic::< $ty >(); }

        #[ test ]
        fn vector_scalar_mul_div() { vector_scalar_mul_div_generic::< $ty >(); }

        #[ test ]
        fn vector_rem() { vector_rem_generic::< $ty >(); }

        #[ test ]
        fn vector_dot_and_mag2() { vector_dot_generic::< $ty >(); }

        #[ test ]
        fn mat_add_sub_row_major()
        {
          mat_add_sub_generic::< $ty, the_module::mat::DescriptorOrderRowMajor >();
        }

        #[ test ]
        fn mat_add_sub_column_major()
        {
          mat_add_sub_generic::< $ty, the_module::mat::DescriptorOrderColumnMajor >();
        }

        #[ test ]
        fn mat_mat_mul_row_major()
        {
          mat_mat_mul_generic::< $ty, the_module::mat::DescriptorOrderRowMajor >();
        }

        #[ test ]
        fn mat_mat_mul_column_major()
        {
          mat_mat_mul_generic::< $ty, the_module::mat::DescriptorOrderColumnMajor >();
        }

        #[ test ]
        fn mat_vec_mul_column_major()
        {
          mat_vec_mul_generic::< $ty, the_module::mat::DescriptorOrderColumnMajor >();
        }
      }
    )*
  };
}

integer_arithmetic_tests!( i32, i64, u32, u64 );

// cross and distance_squared involve subtraction — only safe for signed types.

#[ test ]
fn vec3_cross_i32() { vec3_cross_signed_generic::< i32 >(); }
#[ test ]
fn vec3_cross_i64() { vec3_cross_signed_generic::< i64 >(); }

#[ test ]
fn vector_distance_squared_i32() { vector_distance_squared_signed_generic::< i32 >(); }
#[ test ]
fn vector_distance_squared_i64() { vector_distance_squared_signed_generic::< i64 >(); }

// Neg negates each component — only meaningful for signed types.

#[ test ]
fn vector_neg_i32() { vector_neg_signed_generic::< i32 >(); }
#[ test ]
fn vector_neg_i64() { vector_neg_signed_generic::< i64 >(); }

// `%` preserves the sign of the dividend — only meaningful for signed types.

#[ test ]
fn vector_rem_signed_i32() { vector_rem_signed_generic::< i32 >(); }
#[ test ]
fn vector_rem_signed_i64() { vector_rem_signed_generic::< i64 >(); }

// `/` truncates toward zero — sign handling only meaningful for signed types.

#[ test ]
fn vector_div_signed_i32() { vector_div_signed_generic::< i32 >(); }
#[ test ]
fn vector_div_signed_i64() { vector_div_signed_generic::< i64 >(); }

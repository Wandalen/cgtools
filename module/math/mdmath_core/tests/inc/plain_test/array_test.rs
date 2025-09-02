#![ allow( clippy::uninlined_format_args ) ]

#[ allow( unused_imports ) ]
use super::*;

#[ test ]
fn md_offset_basic()
{
  use the_module::plain::DimOffset;

  // 0D Test case: Single scalar
  let md_size : [ usize ; 0 ] = [];
  let md_index : [ usize ; 0 ] = [];
  let got = md_size.offset( &md_index );
  let exp = 0; // No dimensions, so offset is 0
  assert_eq!( got, exp, "0D test failed" );

  // 1D Test case: Basic test
  let md_size = [ 10 ];
  let md_index = [ 2 ];
  let got = md_size.offset( &md_index );
  let exp = 2; // Only one dimension, so offset is the index itself
  assert_eq!( got, exp, "1D basic test failed" );

  // 1D Test case: Zero index
  let md_index = [ 0 ];
  let got = md_size.offset( &md_index );
  let exp = 0; // Zero index, offset should be 0
  assert_eq!( got, exp, "1D zero index test failed" );

  // 2D Test case: Basic test
  let md_size = [ 10, 100 ];
  let md_index = [ 2, 3 ];
  let got = md_size.offset( &md_index );
  let exp = 2 * 100 + 3; // 2D offset calculation
  assert_eq!( got, exp, "2D basic test failed" );

  // 2D Test case: Zero index
  let md_index = [ 0, 0 ];
  let got = md_size.offset( &md_index );
  let exp = 0; // Zero index, offset should be 0
  assert_eq!( got, exp, "2D zero index test failed" );

  // 3D Basic test
  let md_size = [ 10, 100, 1000 ];
  let md_index = [ 2, 3, 4 ];
  let got = md_size.offset( &md_index );
  let exp = 2 * 100 * 1000 + 3 * 1000 + 4;
  assert_eq!( got, exp, "Basic test failed" );

  // 3D Zero index
  let md_index = [ 0, 0, 0 ];
  let got = md_size.offset( &md_index );
  let exp = 0;
  assert_eq!( got, exp, "Zero index test failed" );

  // 3D Entity
  let md_index = [ 1, 1, 1 ];
  let got = md_size.offset( &md_index );
  let exp = 1000*100 + 1000 + 1;
  assert_eq!( got, exp, "Zero index test failed" );

}

#![ allow( clippy::uninlined_format_args ) ]

#[ allow( unused_imports ) ]
use super::*;

#[ test ]
fn tuple_array_layout_assumptions()
{
  use core::mem::{ align_of_val, size_of_val, offset_of };

  // Define a tuple and an array
  let tuple : ( u8, u8 ) = ( 0, 0 );
  let array : [ u8; 2 ] = [ 0, 0 ];

  // Check size
  let size_tuple = size_of_val( &tuple );
  let size_array = size_of_val( &array );
  println!( "Size: tuple = {}, array = {}", size_tuple, size_array );
  assert_eq!( size_tuple, size_array, "Size should be the same" );

  // Check alignment
  let align_tuple = align_of_val( &tuple );
  let align_array = align_of_val( &array );
  println!( "Alignment: tuple = {}, array = {}", align_tuple, align_array );
  assert_eq!( align_tuple, align_array, "Alignment should be the same" );

  // Check layout of each component
  let align_tuple_0 = align_of_val( &tuple.0 );
  let align_array_0 = align_of_val( &array[ 0 ] );
  println!( "Component 0 alignment: tuple = {}, array = {}", align_tuple_0, align_array_0 );
  assert_eq!( align_tuple_0, align_array_0, "Component 0 alignment should be the same" );

  let align_tuple_1 = align_of_val( &tuple.1 );
  let align_array_1 = align_of_val( &array[ 1 ] );
  println!( "Component 1 alignment: tuple = {}, array = {}", align_tuple_1, align_array_1 );
  assert_eq!( align_tuple_1, align_array_1, "Component 1 alignment should be the same" );

  // Hypothetical offset_of usage
  // Assuming offset_of is available in your environment
  let offset_tuple_0 = offset_of!(( u8, u8 ), 0);
  // let offset_array_0 = offset_of!([ u8; 2 ], [0] );
  let offset_array_0 = 0;
  println!("Component 0 offset: tuple = {}, array = {}", offset_tuple_0, offset_array_0);
  assert_eq!(offset_tuple_0, offset_array_0, "Component 0 offset should be the same");

  let offset_tuple_1 = offset_of!(( u8, u8 ), 1);
  // let offset_array_1 = offset_of!([ u8; 2 ], 1);
  let offset_array_1 = 1;
  println!("Component 1 offset: tuple = {}, array = {}", offset_tuple_1, offset_array_1);
  assert_eq!(offset_tuple_1, offset_array_1, "Component 1 offset should be the same");

  // Uncomment to fail the test and see the output
  // assert!(false);
}

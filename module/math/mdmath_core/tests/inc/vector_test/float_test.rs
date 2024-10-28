use super::*;

#[ allow( unused_imports ) ]
use super::*;

#[ test ]
fn test_all_true()
{
  use the_module::vector::IterExt;

  // Test with all true values (by value)
  let all_true_vec = [ true, true, true ];
  assert!( all_true_vec.iter().copied().all_true(), "Expected all elements to be true" );

  // Test with some false values (by value)
  let some_false_vec = [ true, false, true ];
  assert!( !some_false_vec.iter().copied().all_true(), "Expected not all elements to be true" );

  // Test with all false values (by value)
  let all_false_vec = [ false, false, false ];
  assert!( !all_false_vec.iter().copied().all_true(), "Expected not all elements to be true" );

  // Test with an empty iterator (by value)
  let empty_vec : [ bool; 0 ] = [];
  assert!( empty_vec.iter().copied().all_true(), "Expected all elements to be true for an empty iterator" );

  // Test with all true values (by reference)
  assert!( all_true_vec.iter().all_true(), "Expected all elements to be true" );

  // Test with some false values (by reference)
  assert!( !some_false_vec.iter().all_true(), "Expected not all elements to be true" );

  // Test with all false values (by reference)
  assert!( !all_false_vec.iter().all_true(), "Expected not all elements to be true" );

  // Test with an empty iterator (by reference)
  assert!( empty_vec.iter().all_true(), "Expected all elements to be true for an empty iterator" );

  // Test with all true values (by mutable reference)
  let mut all_true_vec_mut = [ true, true, true ];
  assert!( all_true_vec_mut.iter_mut().all_true(), "Expected all elements to be true" );

  // Test with some false values (by mutable reference)
  let mut some_false_vec_mut = [ true, false, true ];
  assert!( !some_false_vec_mut.iter_mut().all_true(), "Expected not all elements to be true" );

  // Test with all false values (by mutable reference)
  let mut all_false_vec_mut = [ false, false, false ];
  assert!( !all_false_vec_mut.iter_mut().all_true(), "Expected not all elements to be true" );

  // Test with an empty iterator (by mutable reference)
  let mut empty_vec_mut : [ bool; 0 ] = [];
  assert!( empty_vec_mut.iter_mut().all_true(), "Expected all elements to be true for an empty iterator" );
}

#[ test ]
fn test_any_true()
{
  use the_module::vector::IterExt;

  // Test with all true values (by value)
  let all_true_vec = [ true, true, true ];
  assert!( all_true_vec.iter().copied().any_true(), "Expected any element to be true" );

  // Test with some false values (by value)
  let some_false_vec = [ true, false, true ];
  assert!( some_false_vec.iter().copied().any_true(), "Expected any element to be true" );

  // Test with all false values (by value)
  let all_false_vec = [ false, false, false ];
  assert!( !all_false_vec.iter().copied().any_true(), "Expected no elements to be true" );

  // Test with an empty iterator (by value)
  let empty_vec : [ bool; 0 ] = [];
  assert!( !empty_vec.iter().copied().any_true(), "Expected no elements to be true for an empty iterator" );

  // Test with all true values (by reference)
  assert!( all_true_vec.iter().any_true(), "Expected any element to be true" );

  // Test with some false values (by reference)
  assert!( some_false_vec.iter().any_true(), "Expected any element to be true" );

  // Test with all false values (by reference)
  assert!( !all_false_vec.iter().any_true(), "Expected no elements to be true" );

  // Test with an empty iterator (by reference)
  assert!( !empty_vec.iter().any_true(), "Expected no elements to be true for an empty iterator" );

  // Test with all true values (by mutable reference)
  let mut all_true_vec_mut = [ true, true, true ];
  assert!( all_true_vec_mut.iter_mut().any_true(), "Expected any element to be true" );

  // Test with some false values (by mutable reference)
  let mut some_false_vec_mut = [ true, false, true ];
  assert!( some_false_vec_mut.iter_mut().any_true(), "Expected any element to be true" );

  // Test with all false values (by mutable reference)
  let mut all_false_vec_mut = [ false, false, false ];
  assert!( !all_false_vec_mut.iter_mut().any_true(), "Expected no elements to be true" );

  // Test with an empty iterator (by mutable reference)
  let mut empty_vec_mut : [ bool; 0 ] = [];
  assert!( !empty_vec_mut.iter_mut().any_true(), "Expected no elements to be true for an empty iterator" );
}

#[ test ]
fn test_is_nan()
{
  use the_module::vector::IterFloat;

  // Test with a vector containing NaN values
  let vec_with_nan = [ f32::NAN, f32::NAN ];
  assert!( vec_with_nan.iter().is_nan().all( | x | x ), "Expected all elements to be NaN" );

  // Test with a vector without NaN values
  let vec_without_nan = [ 3.0, 4.0 ];
  assert!( !vec_without_nan.iter().is_nan().any( | x | x ), "Expected no elements to be NaN" );

  // Test with a mixed vector
  let mixed_vec = [ 3.0, f32::NAN ];
  assert!( mixed_vec.iter().is_nan().any( | x | x ), "Expected some elements to be NaN" );

  // Test with a mixed vector
  let mixed_vec = [ 3.0, f32::NAN ];
  assert!( mixed_vec.iter().is_nan().any( | x | x ), "Expected some elements to be NaN" );

}
use super::*;

#[ test ]
fn test_const_length_tuple1()
{
  use the_module::ConstLength;
  assert_eq!( < ( i32, ) as ConstLength >::LEN, 1 );
}

#[ test ]
fn test_vector_ref_tuple1()
{
  use the_module::ArrayRef;
  let tuple : (i32,) = (42,);
  let array_ref : &[i32; 1] = tuple.array_ref();
  assert_eq!( array_ref, &[42] );
}

#[ test ]
fn test_vector_mut_tuple1()
{
  use the_module::ArrayMut;
  let mut tuple : (i32,) = (42,);
  {
    let vector_mut : &mut [i32; 1] = tuple.vector_mut();
    vector_mut[0] = 100;
  }
  assert_eq!( tuple, (100,) );
}

#[ test ]
fn test_vector_iter_tuple1()
{
  use the_module::VectorIter;
  let tuple : (i32,) = (42,);
  let mut iter = tuple.vector_iter();
  assert_eq!( iter.next(), Some(&42) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_tuple1()
{
  use the_module::VectorIterMut;
  let mut tuple : (i32,) = (42,);
  {
    let mut iter = tuple.vector_iter_mut();
    if let Some(x) = iter.next()
    {
      *x = 100;
    }
    assert_eq!( iter.next(), None );
  }
  assert_eq!( tuple, (100,) );
}

#[ test ]
fn test_vector_iter_rev_tuple1()
{
  use the_module::VectorIter;
  let tuple : (i32,) = (42,);
  let mut iter = tuple.vector_iter().rev();
  assert_eq!( iter.next(), Some(&42) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_rev_tuple1()
{
  use the_module::VectorIterMut;
  let mut tuple : (i32,) = (42,);
  {
    let mut iter = tuple.vector_iter_mut().rev();
    if let Some(x) = iter.next()
    {
      *x = 100;
    }
    assert_eq!( iter.next(), None );
  }
  assert_eq!( tuple, (100,) );
}

/// Mixed-direction sequence on the single-element mutable iterator: after `.next()` yields the
/// only element, `.next_back()` must not yield it again (and vice versa). Guards the invariant
/// BUG-050 documented for tuple2-4 — a `&mut` iterator must never hand out two live references
/// to the same field — at the arity where the original hand-rolled cursor was replaced by
/// `std::iter::once`.
#[ test ]
fn test_vector_iter_mut_mixed_direction_tuple1()
{
  use the_module::VectorIterMut;

  let mut tuple : ( i32, ) = ( 42, );
  {
    let mut iter = tuple.vector_iter_mut();
    if let Some( x ) = iter.next()
    {
      *x = 100;
    }
    assert_eq!( iter.next_back(), None );
    assert_eq!( iter.next(), None );
  }
  assert_eq!( tuple, ( 100, ) );

  let mut tuple : ( i32, ) = ( 42, );
  {
    let mut iter = tuple.vector_iter_mut();
    if let Some( x ) = iter.next_back()
    {
      *x = 7;
    }
    assert_eq!( iter.next(), None );
    assert_eq!( iter.next_back(), None );
  }
  assert_eq!( tuple, ( 7, ) );
}

/// `ExactSizeIterator::len` and `size_hint` must agree with the fixed arity (1) before
/// iteration and drop to 0 after the element is consumed from either end.
#[ test ]
fn test_vector_iter_size_hint_tuple1()
{
  use the_module::{ VectorIter, VectorIterMut };

  let tuple : ( i32, ) = ( 42, );
  let mut iter = tuple.vector_iter();
  assert_eq!( iter.len(), 1 );
  assert_eq!( iter.size_hint(), ( 1, Some( 1 ) ) );
  iter.next();
  assert_eq!( iter.len(), 0 );
  assert_eq!( iter.size_hint(), ( 0, Some( 0 ) ) );

  let mut tuple : ( i32, ) = ( 42, );
  let mut iter = tuple.vector_iter_mut();
  assert_eq!( iter.len(), 1 );
  iter.next_back();
  assert_eq!( iter.len(), 0 );
}

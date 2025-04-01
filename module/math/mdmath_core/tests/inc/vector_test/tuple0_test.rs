use super::*;

#[ test ]
fn test_const_length_tuple0()
{
  use the_module::ConstLength;
  assert_eq!( <() as ConstLength>::LEN, 0 );
}

#[ test ]
fn test_vector_ref_tuple0()
{
  use the_module::ArrayRef;
  let tuple : () = ();
  let vector_ref : &[ usize; 0 ] = tuple.vector_ref();
  assert_eq!( vector_ref, &[] as &[usize; 0] );
}

#[ test ]
fn test_vector_mut_tuple0()
{
  use the_module::ArrayMut;
  let mut tuple : () = ();
  {
    let vector_mut : &mut [ usize; 0 ] = tuple.vector_mut();
    assert_eq!( vector_mut, &mut [] as &mut [usize; 0] );
  }
  assert_eq!( tuple, () );
}

#[ test ]
fn test_vector_iter_tuple0()
{
  use the_module::VectorIter;
  let tuple : () = ();
  let mut iter = tuple.vector_iter();
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_tuple0()
{
  use the_module::VectorIterMut;
  let mut tuple : () = ();
  {
    let mut iter = tuple.vector_iter_mut();
    assert_eq!( iter.next(), None );
  }
  assert_eq!( tuple, () );
}

#[ test ]
fn test_vector_iter_rev_tuple0()
{
  use the_module::VectorIter;
  let tuple : () = ();
  let mut iter = tuple.vector_iter().rev();
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_rev_tuple0()
{
  use the_module::VectorIterMut;
  let mut tuple : () = ();
  {
    let mut iter = tuple.vector_iter_mut().rev();
    assert_eq!( iter.next(), None );
  }
  assert_eq!( tuple, () );
}

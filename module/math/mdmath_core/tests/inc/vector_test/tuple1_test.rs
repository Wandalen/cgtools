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
  let vector_ref : &[i32; 1] = tuple.vector_ref();
  assert_eq!( vector_ref, &[42] );
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

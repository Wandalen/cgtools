use super::*;

#[ test ]
fn test_const_length_tuple()
{
  use the_module::ConstLength;
  assert_eq!( <( i32, i32 ) as ConstLength>::LEN, 2 );
}

#[ test ]
fn test_vector_ref_tuple()
{
  use the_module::VectorRef;
  let tuple : ( i32, i32 ) = ( 42, 43 );
  let vector_ref : &[ i32; 2 ] = tuple.vector_ref();
  assert_eq!( vector_ref, &[ 42, 43 ] );
}

#[ test ]
fn test_vector_mut_tuple()
{
  use the_module::VectorMut;
  let mut tuple : ( i32, i32 ) = ( 42, 43 );
  {
    let vector_mut : &mut [ i32; 2 ] = tuple.vector_mut();
    vector_mut[ 0 ] = 100;
    vector_mut[ 1 ] = 200;
  }
  assert_eq!( tuple, ( 100, 200 ) );
}

#[ test ]
fn test_vector_iter_tuple()
{
  use the_module::VectorIter;
  let tuple : ( i32, i32 ) = ( 42, 43 );
  let mut iter = tuple.vector_iter();
  assert_eq!( iter.next(), Some( &42 ) );
  assert_eq!( iter.next(), Some( &43 ) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_tuple()
{
  use the_module::VectorIterMut;
  let mut tuple : ( i32, i32 ) = ( 42, 43 );
  {
    let mut iter = tuple.vector_iter_mut();
    if let Some( x ) = iter.next()
    {
      *x = 100;
    }
    if let Some( x ) = iter.next()
    {
      *x = 200;
    }
    assert_eq!( iter.next(), None );
  }
  assert_eq!( tuple, ( 100, 200 ) );
}

#[ test ]
fn test_vector_iter_rev_tuple()
{
  use the_module::VectorIter;
  let tuple : ( i32, i32 ) = ( 42, 43 );
  let mut iter = tuple.vector_iter().rev();
  assert_eq!( iter.next(), Some( &43 ) );
  assert_eq!( iter.next(), Some( &42 ) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_rev_tuple()
{
  use the_module::VectorIterMut;
  let mut tuple : ( i32, i32 ) = ( 42, 43 );
  {
    let mut iter = tuple.vector_iter_mut().rev();
    if let Some( x ) = iter.next()
    {
      *x = 200;
    }
    if let Some( x ) = iter.next()
    {
      *x = 100;
    }
    assert_eq!( iter.next(), None );
  }
  assert_eq!( tuple, ( 100, 200 ) );
}

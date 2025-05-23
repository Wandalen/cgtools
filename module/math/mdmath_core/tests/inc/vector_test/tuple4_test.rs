use super::*;

#[ test ]
fn test_const_length_tuple4()
{
  use the_module::ConstLength;
  assert_eq!( <( i32, i32, i32, i32 ) as ConstLength>::LEN, 4 );
}

#[ test ]
fn test_vector_ref_tuple4()
{
  use the_module::ArrayRef;
  let tuple : ( i32, i32, i32, i32 ) = ( 42, 43, 44, 45 );
  let array_ref : &[ i32; 4 ] = tuple.array_ref();
  assert_eq!( array_ref, &[ 42, 43, 44, 45 ] );
}

#[ test ]
fn test_vector_mut_tuple4()
{
  use the_module::ArrayMut;
  let mut tuple : ( i32, i32, i32, i32 ) = ( 42, 43, 44, 45 );
  {
    let vector_mut : &mut [ i32; 4 ] = tuple.vector_mut();
    vector_mut[ 0 ] = 100;
    vector_mut[ 1 ] = 200;
    vector_mut[ 2 ] = 300;
    vector_mut[ 3 ] = 400;
  }
  assert_eq!( tuple, ( 100, 200, 300, 400 ) );
}

#[ test ]
fn test_vector_iter_tuple4()
{
  use the_module::VectorIter;
  let tuple : ( i32, i32, i32, i32 ) = ( 42, 43, 44, 45 );
  let mut iter = tuple.vector_iter();
  assert_eq!( iter.next(), Some( &42 ) );
  assert_eq!( iter.next(), Some( &43 ) );
  assert_eq!( iter.next(), Some( &44 ) );
  assert_eq!( iter.next(), Some( &45 ) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_tuple4()
{
  use the_module::VectorIterMut;
  let mut tuple : ( i32, i32, i32, i32 ) = ( 42, 43, 44, 45 );
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
    if let Some( x ) = iter.next()
    {
      *x = 300;
    }
    if let Some( x ) = iter.next()
    {
      *x = 400;
    }
    assert_eq!( iter.next(), None );
  }
  assert_eq!( tuple, ( 100, 200, 300, 400 ) );
}

#[ test ]
fn test_vector_iter_rev_tuple4()
{
  use the_module::VectorIter;
  let tuple : ( i32, i32, i32, i32 ) = ( 42, 43, 44, 45 );
  let mut iter = tuple.vector_iter().rev();
  assert_eq!( iter.next(), Some( &45 ) );
  assert_eq!( iter.next(), Some( &44 ) );
  assert_eq!( iter.next(), Some( &43 ) );
  assert_eq!( iter.next(), Some( &42 ) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_rev_tuple4()
{
  use the_module::VectorIterMut;
  let mut tuple : ( i32, i32, i32, i32 ) = ( 42, 43, 44, 45 );
  {
    let mut iter = tuple.vector_iter_mut().rev();
    if let Some( x ) = iter.next()
    {
      *x = 400;
    }
    if let Some( x ) = iter.next()
    {
      *x = 300;
    }
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
  assert_eq!( tuple, ( 100, 200, 300, 400 ) );
}

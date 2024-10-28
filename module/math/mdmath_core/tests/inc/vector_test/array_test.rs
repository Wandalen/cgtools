use super::*;

#[ test ]
fn test_const_length()
{
  use the_module::ConstLength;
  assert_eq!( <[ i32; 0 ] as ConstLength>::LEN, 0 );
  assert_eq!( <[ i32; 1 ] as ConstLength>::LEN, 1 );
  assert_eq!( <[ i32; 3 ] as ConstLength>::LEN, 3 );
}

#[ test ]
fn test_vector_ref()
{
  use the_module::VectorRef;
  let array : [ i32; 0 ] = [];
  let vector_ref : &[ i32; 0 ] = array.vector_ref();
  assert_eq!( vector_ref, &[] as &[ i32; 0 ] );

  let array : [ i32; 1 ] = [ 42 ];
  let vector_ref : &[ i32; 1 ] = array.vector_ref();
  assert_eq!( vector_ref, &[ 42 ] );

  let array : [ i32; 3 ] = [ 1, 2, 3 ];
  let vector_ref : &[ i32; 3 ] = array.vector_ref();
  assert_eq!( vector_ref, &[ 1, 2, 3 ] );
}

#[ test ]
fn test_vector_mut()
{
  use the_module::VectorMut;
  let mut array : [ i32; 0 ] = [];
  {
    let vector_mut : &mut [ i32; 0 ] = array.vector_mut();
    assert_eq!( vector_mut, &mut [] as &mut [ i32; 0 ] );
  }

  let mut array : [ i32; 1 ] = [ 42 ];
  {
    let vector_mut : &mut [ i32; 1 ] = array.vector_mut();
    vector_mut[ 0 ] = 100;
  }
  assert_eq!( array, [ 100 ] );

  let mut array : [ i32; 3 ] = [ 1, 2, 3 ];
  {
    let vector_mut : &mut [ i32; 3 ] = array.vector_mut();
    vector_mut[ 0 ] = 10;
    vector_mut[ 1 ] = 20;
    vector_mut[ 2 ] = 30;
  }
  assert_eq!( array, [ 10, 20, 30 ] );
}

#[ test ]
fn test_vector_iter()
{
  use the_module::VectorIter;
  let array : [ i32; 0 ] = [];
  let mut iter = array.vector_iter();
  assert_eq!( iter.next(), None );

  let array : [ i32; 1 ] = [ 42 ];
  let mut iter = array.vector_iter();
  assert_eq!( iter.next(), Some( &42 ) );
  assert_eq!( iter.next(), None );

  let array : [ i32; 3 ] = [ 1, 2, 3 ];
  let mut iter = array.vector_iter();
  assert_eq!( iter.next(), Some( &1 ) );
  assert_eq!( iter.next(), Some( &2 ) );
  assert_eq!( iter.next(), Some( &3 ) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut()
{
  use the_module::VectorIterMut;
  let mut array : [ i32; 0 ] = [];
  {
    let mut iter = array.vector_iter_mut();
    assert_eq!( iter.next(), None );
  }

  let mut array : [ i32; 1 ] = [ 42 ];
  {
    let mut iter = array.vector_iter_mut();
    if let Some( x ) = iter.next()
    {
      *x = 100;
    }
    assert_eq!( iter.next(), None );
  }
  assert_eq!( array, [ 100 ] );

  let mut array : [ i32; 3 ] = [ 1, 2, 3 ];
  {
    let mut iter = array.vector_iter_mut();
    if let Some( x ) = iter.next()
    {
      *x = 10;
    }
    if let Some( x ) = iter.next()
    {
      *x = 20;
    }
    if let Some( x ) = iter.next()
    {
      *x = 30;
    }
    assert_eq!( iter.next(), None );
  }
  assert_eq!( array, [ 10, 20, 30 ] );
}
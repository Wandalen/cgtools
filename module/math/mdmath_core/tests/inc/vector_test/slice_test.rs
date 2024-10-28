use super::*;

#[ test ]
fn test_vector_ref_slice()
{
  use the_module::VectorRef;
  let slice : &[ i32 ] = &[];
  let vector_ref : &[ i32; 0 ] = slice.vector_ref();
  assert_eq!( vector_ref, &[] as &[ i32; 0 ] );

  let slice : &[ i32 ] = &[ 42 ];
  let vector_ref : &[ i32; 1 ] = slice.vector_ref();
  assert_eq!( vector_ref, &[ 42 ] );

  let slice : &[ i32 ] = &[ 1, 2, 3 ];
  let vector_ref : &[ i32; 3 ] = slice.vector_ref();
  assert_eq!( vector_ref, &[ 1, 2, 3 ] );
}

#[ test ]
fn test_vector_mut_slice()
{
  use the_module::VectorMut;
  let slice : &mut [ i32 ] = &mut [];
  {
    let vector_mut : &mut [ i32; 0 ] = slice.vector_mut();
    assert_eq!( vector_mut, &mut [] as &mut [ i32; 0 ] );
  }

  let slice : &mut [ i32 ] = &mut [ 42 ];
  {
    let vector_mut : &mut [ i32; 1 ] = slice.vector_mut();
    vector_mut[ 0 ] = 100;
  }
  assert_eq!( slice, &[ 100 ] );

  let slice : &mut [ i32 ] = &mut [ 1, 2, 3 ];
  {
    let vector_mut : &mut [ i32; 3 ] = slice.vector_mut();
    vector_mut[ 0 ] = 10;
    vector_mut[ 1 ] = 20;
    vector_mut[ 2 ] = 30;
  }
  assert_eq!( slice, &[ 10, 20, 30 ] );
}

#[ test ]
fn test_vector_iter_slice()
{
  use the_module::VectorIter;
  let slice : &[ i32 ] = &[];
  let mut iter = <[ i32 ] as VectorIter< i32, 0 >>::vector_iter( slice );
  assert_eq!( iter.next(), None );

  let slice : &[ i32 ] = &[ 42 ];
  let mut iter = <[ i32 ] as VectorIter< i32, 1 >>::vector_iter( slice );
  assert_eq!( iter.next(), Some( &42 ) );
  assert_eq!( iter.next(), None );

  let slice : &[ i32 ] = &[ 1, 2, 3 ];
  let mut iter = <[ i32 ] as VectorIter< i32, 3 >>::vector_iter( slice );
  assert_eq!( iter.next(), Some( &1 ) );
  assert_eq!( iter.next(), Some( &2 ) );
  assert_eq!( iter.next(), Some( &3 ) );
  assert_eq!( iter.next(), None );
}

#[ test ]
fn test_vector_iter_mut_slice()
{
  use the_module::VectorIterMut;
  let slice : &mut [ i32 ] = &mut [];
  {
    let mut iter = <[ i32 ] as VectorIterMut< i32, 0 >>::vector_iter_mut( slice );
    assert_eq!( iter.next(), None );
  }

  let slice : &mut [ i32 ] = &mut [ 42 ];
  {
    let mut iter = <[ i32 ] as VectorIterMut< i32, 1 >>::vector_iter_mut( slice );
    if let Some( x ) = iter.next()
    {
      *x = 100;
    }
    assert_eq!( iter.next(), None );
  }
  assert_eq!( slice, &[ 100 ] );

  let slice : &mut [ i32 ] = &mut [ 1, 2, 3 ];
  {
    let mut iter = <[ i32 ] as VectorIterMut< i32, 3 >>::vector_iter_mut( slice );
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
  assert_eq!( slice, &[ 10, 20, 30 ] );
}

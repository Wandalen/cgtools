use super::*;

// = 0

impl Collection for ()
{
  // We use usize as the Scalar type instead of () because many numeric traits
  // are not implemented for the unit type. This allows for more flexibility
  // and compatibility with other numeric operations.
  type Scalar = usize;
}

impl ConstLength for ()
{
  const LEN : usize = 0;
}

impl< E > IntoArray< E, 0 > for ()
{
  #[ inline ]
  fn into_array( self ) -> [ E ; 0 ]
  {
    []
  }
}

impl ArrayRef< usize, 0 > for ()
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ usize ; 0 ]
  {
    // Return an empty array of usize. This is safe because an empty array
    // of any type has the same memory representation.
    &[]
  }
}

impl ArrayMut< usize, 0 > for ()
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ usize ; 0 ]
  {
    // Return a mutable reference to an empty array of usize.
    &mut []
  }
}

// Tuple0Iter represents an iterator over a tuple with 0 elements.
// It's always empty and never yields any items.
#[ derive( Clone ) ]
struct Tuple0Iter< 'tuple_ref >
{
  // PhantomData is used to make the struct generic over 'tuple_ref without storing any data.
  _phantom : std::marker::PhantomData< &'tuple_ref () >,
}

impl< 'tuple_ref > Iterator for Tuple0Iter< 'tuple_ref >
{
  type Item = &'tuple_ref usize;

  fn next( &mut self ) -> Option< Self::Item >
  {
    // Always returns None as there are no elements to iterate over.
    None
  }

  fn size_hint( &self ) -> ( usize, Option< usize > )
  {
    // The iterator is always empty, so both lower and upper bounds are 0.
    ( 0, Some( 0 ) )
  }
}

// Implement ExactSizeIterator as we always know the exact number of iterations (0).
impl< 'tuple_ref > ExactSizeIterator for Tuple0Iter< 'tuple_ref > {}

// Implement DoubleEndedIterator as we can iterate from both ends (although it's always empty).
impl< 'tuple_ref > DoubleEndedIterator for Tuple0Iter< 'tuple_ref >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    // Always returns None as there are no elements to iterate over.
    None
  }
}

// Tuple0IterMut represents a mutable iterator over a tuple with 0 elements.
// It's always empty and never yields any items.
struct Tuple0IterMut< 'tuple_ref >
{
  // PhantomData is used to make the struct generic over 'tuple_ref without storing any data.
  _phantom : std::marker::PhantomData< &'tuple_ref mut () >,
}

impl< 'tuple_ref > Iterator for Tuple0IterMut< 'tuple_ref >
{
  type Item = &'tuple_ref mut usize;

  fn next( &mut self ) -> Option< Self::Item >
  {
    // Always returns None as there are no elements to iterate over.
    None
  }

  fn size_hint( &self ) -> ( usize, Option< usize > )
  {
    // The iterator is always empty, so both lower and upper bounds are 0.
    ( 0, Some( 0 ) )
  }
}

// Implement ExactSizeIterator as we always know the exact number of iterations (0).
impl< 'tuple_ref > ExactSizeIterator for Tuple0IterMut< 'tuple_ref > {}

// Implement DoubleEndedIterator as we can iterate from both ends (although it's always empty).
impl< 'tuple_ref > DoubleEndedIterator for Tuple0IterMut< 'tuple_ref >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    // Always returns None as there are no elements to iterate over.
    None
  }
}

impl VectorIter< usize, 0 > for ()
{
  fn vector_iter< 'tuple_ref >( &'tuple_ref self ) -> impl VectorIteratorRef< 'tuple_ref, &'tuple_ref usize >
  where
    usize: 'tuple_ref,
  {
    // Return an empty iterator
    Tuple0Iter
    {
      _phantom : std::marker::PhantomData,
    }
  }
}

impl VectorIterMut< usize, 0 > for ()
{
  fn vector_iter_mut< 'tuple_ref >( &'tuple_ref mut self ) -> impl VectorIterator< 'tuple_ref, &'tuple_ref mut usize >
  where
    usize: 'tuple_ref,
  {
    // Return an empty mutable iterator
    Tuple0IterMut
    {
      _phantom : std::marker::PhantomData,
    }
  }
}

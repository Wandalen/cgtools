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

impl VectorRef< usize, 0 > for ()
{
  #[ inline( always ) ]
  fn vector_ref( &self ) -> &[ usize ; 0 ]
  {
    // Return an empty array of usize. This is safe because an empty array
    // of any type has the same memory representation.
    &[]
  }
}

impl VectorMut< usize, 0 > for ()
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
struct Tuple0Iter< 'a >
{
  // PhantomData is used to make the struct generic over 'a without storing any data.
  _phantom : std::marker::PhantomData< &'a () >,
}

impl< 'a > Iterator for Tuple0Iter< 'a >
{
  type Item = &'a usize;

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
impl< 'a > ExactSizeIterator for Tuple0Iter< 'a > {}

// Implement DoubleEndedIterator as we can iterate from both ends (although it's always empty).
impl< 'a > DoubleEndedIterator for Tuple0Iter< 'a >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    // Always returns None as there are no elements to iterate over.
    None
  }
}

// Tuple0IterMut represents a mutable iterator over a tuple with 0 elements.
// It's always empty and never yields any items.
struct Tuple0IterMut< 'a >
{
  // PhantomData is used to make the struct generic over 'a without storing any data.
  _phantom : std::marker::PhantomData< &'a mut () >,
}

impl< 'a > Iterator for Tuple0IterMut< 'a >
{
  type Item = &'a mut usize;

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
impl< 'a > ExactSizeIterator for Tuple0IterMut< 'a > {}

// Implement DoubleEndedIterator as we can iterate from both ends (although it's always empty).
impl< 'a > DoubleEndedIterator for Tuple0IterMut< 'a >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    // Always returns None as there are no elements to iterate over.
    None
  }
}

impl VectorIter< usize, 0 > for ()
{
  fn vector_iter< 'a >( &'a self ) -> impl VectorIteratorRef< 'a, &'a usize >
  where
    usize: 'a,
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
  fn vector_iter_mut< 'a >( &'a mut self ) -> impl VectorIterator< 'a, &'a mut usize >
  where
    usize: 'a,
  {
    // Return an empty mutable iterator
    Tuple0IterMut
    {
      _phantom : std::marker::PhantomData,
    }
  }
}

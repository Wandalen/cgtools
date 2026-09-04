use crate::{ Collection, ConstLength, IntoArray, ArrayRef, ArrayMut, Ix };
use ::ndarray::{ Ix0, Ix1, Ix2, Ix3, Ix4, Dimension };

// = 0

impl Collection for Ix0
{
  type Scalar = usize;
}

impl ConstLength for Ix0
{
  const LEN : usize = 0;
}

impl IntoArray< usize, 0 > for Ix0
{
  #[ inline ]
  fn into_array( self ) -> [ usize ; 0 ]
  {
    []
  }
}

impl ArrayRef< usize, 0 > for Ix0
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ usize ; 0 ]
  {
    &[]
  }
}

impl ArrayMut< usize, 0 > for Ix0
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ usize ; 0 ]
  {
    &mut []
  }
}

// = 1

impl Collection for Ix1
{
  type Scalar = Ix;
}

impl ConstLength for Ix1
{
  const LEN : usize = 1;
}

impl IntoArray< usize, 1 > for Ix1
{
  #[ inline ]
  fn into_array( self ) -> [ usize ; 1 ]
  {
    [ self[ 0 ] ]
  }
}

impl ArrayRef< usize, 1 > for Ix1
{
  // Fix(BUG-449): replaced an unsafe raw-pointer cast with `ndarray::Dimension::slice()` (a
  // safe, public trait method returning `&[Ix]`) plus std's checked
  // `TryFrom<&[T]> for &[T;N]`.
  // Root cause: the previous unsafe cast from `&Ix1` (`ndarray::Dim<[usize;1]>`) to
  // `&[usize;1]` was justified only by a `debug_assert_eq!` of `size_of_val`/`align_of_val` --
  // that proves the two types share total size and alignment, but proves nothing about field
  // order or padding, which is the actual property a same-layout cast depends on (mismatched
  // field order is instant undefined behavior, not merely a wrong value). The assertion is
  // also compiled out entirely in release builds, so a future `ndarray` version changing
  // `Dim`'s internal layout would silently produce UB with zero runtime signal in exactly the
  // build profile most likely to ship.
  // Pitfall: `size_of`/`align_of` equality is necessary but nowhere near sufficient to justify
  // an unsafe same-layout pointer-cast between two independently-defined types -- when the
  // source type already exposes a safe accessor to its own fields (here: `Dimension::slice()`),
  // prefer a safe conversion built on that accessor over an unsafe cast, even one guarded by a
  // runtime assertion.
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ usize ; 1 ]
  {
    < &[ usize ; 1 ] >::try_from( self.slice() ).expect( "Ix1::slice() always returns exactly 1 element" )
  }
}

impl ArrayMut< usize, 1 > for Ix1
{
  // Fix(BUG-449): see `ArrayRef::array_ref`'s own BUG-449 comment above (same root cause and
  // fix, mirrored here via `Dimension::slice_mut()` + `TryFrom<&mut [T]> for &mut [T;N]`).
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ usize ; 1 ]
  {
    < &mut [ usize ; 1 ] >::try_from( self.slice_mut() ).expect( "Ix1::slice_mut() always returns exactly 1 element" )
  }
}

// = 2

impl Collection for Ix2
{
  type Scalar = Ix;
}

impl ConstLength for Ix2
{
  const LEN : usize = 2;
}

impl IntoArray< usize, 2 > for Ix2
{
  #[ inline ]
  fn into_array( self ) -> [ usize ; 2 ]
  {
    [ self[ 0 ], self[ 1 ] ]
  }
}

impl ArrayRef< usize, 2 > for Ix2
{
  // Fix(BUG-449): see `Ix1`'s `ArrayRef::array_ref` BUG-449 comment above (same root cause and
  // fix, generalized to `N = 2`).
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ usize ; 2 ]
  {
    < &[ usize ; 2 ] >::try_from( self.slice() ).expect( "Ix2::slice() always returns exactly 2 elements" )
  }
}

impl ArrayMut< usize, 2 > for Ix2
{
  // Fix(BUG-449): see `Ix1`'s `ArrayRef::array_ref` BUG-449 comment above (same root cause and
  // fix, generalized to `N = 2`).
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ usize ; 2 ]
  {
    < &mut [ usize ; 2 ] >::try_from( self.slice_mut() ).expect( "Ix2::slice_mut() always returns exactly 2 elements" )
  }
}

// = 3

impl Collection for Ix3
{
  type Scalar = Ix;
}

impl ConstLength for Ix3
{
  const LEN : usize = 3;
}

impl IntoArray< usize, 3 > for Ix3
{
  #[ inline ]
  fn into_array( self ) -> [ usize ; 3 ]
  {
    [ self[ 0 ], self[ 1 ], self[ 2 ] ]
  }
}

impl ArrayRef< usize, 3 > for Ix3
{
  // Fix(BUG-449): see `Ix1`'s `ArrayRef::array_ref` BUG-449 comment above (same root cause and
  // fix, generalized to `N = 3`).
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ usize ; 3 ]
  {
    < &[ usize ; 3 ] >::try_from( self.slice() ).expect( "Ix3::slice() always returns exactly 3 elements" )
  }
}

impl ArrayMut< usize, 3 > for Ix3
{
  // Fix(BUG-449): see `Ix1`'s `ArrayRef::array_ref` BUG-449 comment above (same root cause and
  // fix, generalized to `N = 3`).
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ usize ; 3 ]
  {
    < &mut [ usize ; 3 ] >::try_from( self.slice_mut() ).expect( "Ix3::slice_mut() always returns exactly 3 elements" )
  }
}

// = 4

impl Collection for Ix4
{
  type Scalar = Ix;
}

impl ConstLength for Ix4
{
  const LEN : usize = 4;
}

impl IntoArray< usize, 4 > for Ix4
{
  #[ inline ]
  fn into_array( self ) -> [ usize ; 4 ]
  {
    [ self[ 0 ], self[ 1 ], self[ 2 ], self[ 3 ] ]
  }
}

impl ArrayRef< usize, 4 > for Ix4
{
  // Fix(BUG-449): see `Ix1`'s `ArrayRef::array_ref` BUG-449 comment above (same root cause and
  // fix, generalized to `N = 4`).
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ usize ; 4 ]
  {
    < &[ usize ; 4 ] >::try_from( self.slice() ).expect( "Ix4::slice() always returns exactly 4 elements" )
  }
}

impl ArrayMut< usize, 4 > for Ix4
{
  // Fix(BUG-449): see `Ix1`'s `ArrayRef::array_ref` BUG-449 comment above (same root cause and
  // fix, generalized to `N = 4`).
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ usize ; 4 ]
  {
    < &mut [ usize ; 4 ] >::try_from( self.slice_mut() ).expect( "Ix4::slice_mut() always returns exactly 4 elements" )
  }
}

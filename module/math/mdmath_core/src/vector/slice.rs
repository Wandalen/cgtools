use super::*;

impl< E > Collection for [ E ]
{
  type Scalar = E;
}

// Converted implementation using unwrap_or_else with panic! to avoid the Debug requirement
impl< E, const N : usize > IntoArray< E, N > for &[ E ]
where
  [ E ; N ] : for< 'data > TryFrom< &'data [ E ] >
{
  #[ inline ]
  fn into_array( self ) -> [ E ; N ]
  {
    self.try_into().unwrap_or_else
    (
      | _ | panic!( "Slice length does not match array length : {} != {}", self.len(), N )
    )
  }
}

impl< E, const N : usize > ArrayRef< E, N > for [ E ]
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ E ; N ]
  {
    assert!( self.len() >= N, "Slice must have at least {} element", N );
    // SAFETY: This is safe if the slice has at least 1 element.
    #[ allow( unsafe_code ) ]
    unsafe { &*( self.as_ptr() as *const [ E ; N ] ) }
  }
}

impl< E, const N : usize > ArrayRef< E, N > for &[ E ]
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ E ; N ]
  {
    assert!( ( *self ).len() >= N, "Slice must have at least {} element", N );
    // SAFETY: This is safe if the slice has at least 1 element.
    #[ allow( unsafe_code ) ]
    unsafe { &*( ( *self ).as_ptr() as *const [ E ; N ] ) }
  }
}

impl< E, const N : usize > ArrayMut< E, N > for [ E ]
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ E ; N ]
  {
    assert!( self.len() >= N, "Slice must have at least {} element", N );
    // Fix(BUG-054): as_ptr() carries only SharedReadOnly provenance; casting
    // it to *mut and retagging Unique is UB under Stacked Borrows even
    // though the outer reference is &mut. as_mut_ptr() retags Unique first.
    // Root cause: copy-pasted from the immutable array_ref() sibling above
    // without switching as_ptr() -> as_mut_ptr(). Pitfall: a *mut cast alone
    // never upgrades a pointer's borrow provenance — the source accessor
    // must already be the mutable one.
    // SAFETY: This is safe if the slice has at least N element.
    #[ allow( unsafe_code ) ]
    unsafe { &mut *( self.as_mut_ptr() as *mut [ E ; N ] ) }
  }
}

impl< E, const N : usize > VectorIter< E, N > for [ E ]
{
  fn vector_iter< 'data >( &'data self ) -> impl VectorIteratorRef< 'data, &'data E >
  where
    E : 'data,
  {
    assert!( self.len() >= N, "Slice must have at least {} elements", N );
    <[ E ]>::iter( self ).take( N )
  }
}

impl< E, const N : usize > VectorIterMut< E, N > for [ E ]
{
  fn vector_iter_mut< 'data >( &'data mut self ) -> impl VectorIterator< 'data, &'data mut E >
  where
    E : 'data,
  {
    assert!( self.len() >= N, "Slice must have at least {} elements", N );
    <[ E ]>::iter_mut( self )
  }
}

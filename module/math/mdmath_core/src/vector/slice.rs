use super::{Collection, IntoArray, ArrayRef, ArrayMut, VectorIter, VectorIteratorRef, VectorIterMut, VectorIterator};

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
    assert!( self.len() >= N, "Slice must have at least {N} element" );
    // SAFETY: This is safe if the slice has at least 1 element.
    #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
    unsafe { &*self.as_ptr().cast::<[ E ; N ]>() }
  }
}

impl< E, const N : usize > ArrayRef< E, N > for &[ E ]
{
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ E ; N ]
  {
    assert!( ( *self ).len() >= N, "Slice must have at least {N} element" );
    // SAFETY: This is safe if the slice has at least 1 element.
    #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
    unsafe { &*( *self ).as_ptr().cast::<[ E ; N ]>() }
  }
}

impl< E, const N : usize > ArrayMut< E, N > for [ E ]
{
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ E ; N ]
  {
    assert!( self.len() >= N, "Slice must have at least {N} element" );
    // Fix(BUG-054): as_ptr() carries only SharedReadOnly provenance; casting
    // it to *mut and retagging Unique is UB under Stacked Borrows even
    // though the outer reference is &mut. as_mut_ptr() retags Unique first.
    // Root cause: copy-pasted from the immutable array_ref() sibling above
    // without switching as_ptr() -> as_mut_ptr(). Pitfall: a *mut cast alone
    // never upgrades a pointer's borrow provenance — the source accessor
    // must already be the mutable one.
    // SAFETY: This is safe if the slice has at least N element.
    #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
    unsafe { &mut *self.as_mut_ptr().cast::<[ E ; N ]>() }
  }
}

impl< E, const N : usize > VectorIter< E, N > for [ E ]
{
  #[ inline ]
  fn vector_iter< 'data >( &'data self ) -> impl VectorIteratorRef< 'data, &'data E >
  where
    E : 'data,
  {
    assert!( self.len() >= N, "Slice must have at least {N} elements" );
    <[ E ]>::iter( self ).take( N )
  }
}

impl< E, const N : usize > VectorIterMut< E, N > for [ E ]
{
  // Fix(BUG-123): added `.take( N )`, matching the `VectorIter::vector_iter` impl above.
  // Root cause: both methods assert `self.len() >= N` (not `== N`), establishing a "treat the
  // first N elements of a possibly-longer slice as the logical vector" contract — the same
  // contract `array_ref`/`vector_mut` enforce via their `[E;N]` pointer casts. `vector_iter`
  // upholds it with `.take(N)`; `vector_iter_mut` returned the full `IterMut` unbounded,
  // silently letting a mutation through this trait touch elements at index >= N whenever the
  // backing slice is longer than N.
  // Pitfall: an `>=`-style length assertion documents "first N of possibly more" — every
  // accessor built on it must independently bound its own traversal to N; matching one
  // sibling method's `.take(N)` is not evidence the other methods also apply it.
  #[ inline ]
  fn vector_iter_mut< 'data >( &'data mut self ) -> impl VectorIterator< 'data, &'data mut E >
  where
    E : 'data,
  {
    assert!( self.len() >= N, "Slice must have at least {N} elements" );
    <[ E ]>::iter_mut( self ).take( N )
  }
}

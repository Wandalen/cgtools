use core::mem::{ size_of, align_of };

use super::{Collection, ConstLength, IntoArray, ArrayRef, ArrayMut, VectorIter, VectorIteratorRef, VectorIterMut, VectorIterator};

// Fix(BUG-449): compile-time layout proof shared by `ArrayRef::array_ref`/`ArrayMut::vector_mut`
// below -- see `tuple2.rs`'s `assert_tuple2_array_layout` for the full root-cause/pitfall
// writeup (identical mechanism, generalized to 3 fields).
const fn assert_tuple3_array_layout< E >()
{
  assert!( size_of::< ( E, E, E ) >() == size_of::< [ E ; 3 ] >(), "(E,E,E) and [E;3] must have the same size" );
  assert!( align_of::< ( E, E, E ) >() == align_of::< [ E ; 3 ] >(), "(E,E,E) and [E;3] must have the same alignment" );
  assert!( core::mem::offset_of!( ( E, E, E ), 0 ) == 0, "field 0 must be at byte offset 0" );
  assert!( core::mem::offset_of!( ( E, E, E ), 1 ) == size_of::< E >(), "field 1 must immediately follow field 0 with no padding" );
  assert!( core::mem::offset_of!( ( E, E, E ), 2 ) == 2 * size_of::< E >(), "field 2 must immediately follow field 1 with no padding" );
}

// = 3

impl< E > Collection for ( E, E, E )
{
  type Scalar = E;
}

impl< E > ConstLength for ( E, E, E )
{
  const LEN : usize = 3;
}

impl< E > IntoArray< E, 3 > for ( E, E, E )
{
  #[ inline ]
  fn into_array( self ) -> [ E ; 3 ]
  {
    [ self.0, self.1, self.2 ]
  }
}

impl< E > ArrayRef< E, 3 > for ( E, E, E )
{
  // Fix(BUG-449): replaced the runtime-only, debug-only `debug_assert_eq!` of *total* size/align
  // with an unconditional, compile-time proof (`assert_tuple3_array_layout`) that also checks
  // per-field byte offsets -- see `tuple2.rs`'s identical BUG-449 fix for the full writeup.
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ E ; 3 ]
  {
    const { assert_tuple3_array_layout::< E >() };

    // SAFETY: the compile-time check above proves, for this concrete `E`, that `(E,E,E)` and
    // `[E;3]` share total size, alignment, and per-field byte offsets -- the exact and complete
    // set of properties this reinterpret-cast depends on. The resulting reference's lifetime is
    // tied to `self`'s via the raw-pointer-cast-then-reborrow pattern, so it cannot outlive the
    // data it points to.
    #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
    unsafe { &*( std::ptr::from_ref::< ( E, E, E ) >( self ).cast::< [ E ; 3 ] >() ) }
  }
}

impl< E > ArrayMut< E, 3 > for ( E, E, E )
{
  // Fix(BUG-449): see `ArrayRef::array_ref`'s own BUG-449 comment above (same root cause and
  // fix, mirrored here for the mutable accessor).
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ E ; 3 ]
  {
    const { assert_tuple3_array_layout::< E >() };

    // SAFETY: see `ArrayRef::array_ref` above -- same compile-time proof, mutable variant.
    #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
    unsafe { &mut *( std::ptr::from_mut::< ( E, E, E ) >( self ).cast::< [ E ; 3 ] >() ) }
  }
}

// Fix(BUG-122): `index : usize` was shared between `next()` and `next_back()`, whose match
// arms were hardcoded per-direction — after any `next()` call, `next_back()` reinterpreted
// the resulting `index` as if it were counted from the back, yielding the wrong field (or,
// for the tuple2 sibling, the same field twice while dropping the other entirely) instead of
// the true back of the remaining range.
// Root cause: same shared-single-cursor shape as the already-fixed `Tuple3IterMut` (BUG-050),
// just never itself updated when that fix landed — aliasing `&E` is safe here, so the
// consequence is wrong values rather than UB, but the logic defect is identical.
// Pitfall: a hand-rolled `DoubleEndedIterator` needs independent front/back cursors (mirrors
// `core::slice::Iter`) even when the yielded references are shared and aliasing-safe — a
// single shared counter is a correctness bug, not just a soundness one, under mixed
// `.next()`/`.next_back()` sequences; pure-forward or pure-`.rev()` alone cannot catch it.
#[ derive( Clone ) ]
struct Tuple3Iter< 'tuple_ref, E >
{
  tuple : &'tuple_ref ( E, E, E ),
  front : usize,
  back : usize,
}

impl< 'tuple_ref, E > Iterator for Tuple3Iter< 'tuple_ref, E >
{
  type Item = &'tuple_ref E;

  fn next( &mut self ) -> Option< Self::Item >
  {
    if self.front >= self.back
    {
      return None;
    }

    let index = self.front;
    self.front += 1;

    match index {
      0 => Some( &self.tuple.0 ),
      1 => Some( &self.tuple.1 ),
      2 => Some( &self.tuple.2 ),
      _ => unreachable!(),
    }
  }

  fn size_hint( &self ) -> ( usize, Option< usize > )
  {
    let remaining = self.back - self.front;
    ( remaining, Some( remaining ) )
  }
}

impl< E > ExactSizeIterator for Tuple3Iter< '_, E > {}

impl< E > DoubleEndedIterator for Tuple3Iter< '_, E >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    if self.front >= self.back
    {
      return None;
    }

    self.back -= 1;

    match self.back {
      0 => Some( &self.tuple.0 ),
      1 => Some( &self.tuple.1 ),
      2 => Some( &self.tuple.2 ),
      _ => unreachable!(),
    }
  }
}

// Fix(BUG-050): `index : usize` was shared between `next()` and `next_back()`, whose match
// arms were hardcoded per-direction — mixing the two calls on one iterator (e.g. two `.next()`
// then one `.next_back()`) re-yielded an already-returned tuple field as a second
// simultaneously-live `&mut E` reference instead of reaching the untouched one.
// Root cause: copy-pasted from the immutable `Tuple3Iter` above (where aliasing `&E` is
// harmless) into a `&mut` context without redesigning the cursor for unique-borrow safety.
// Pitfall: a hand-rolled `DoubleEndedIterator` yielding `&mut` references needs independent
// front/back cursors (mirrors `core::slice::IterMut`), never a single shared counter — always
// test a mixed `.next()`/`.next_back()` sequence, not just pure-forward or pure-`.rev()`.
struct Tuple3IterMut< 'tuple_ref, E >
{
  tuple : &'tuple_ref mut ( E, E, E ),
  front : usize,
  back : usize,
}

impl< 'tuple_ref, E > Iterator for Tuple3IterMut< 'tuple_ref, E >
{
  type Item = &'tuple_ref mut E;

  fn next( &mut self ) -> Option< Self::Item >
  {
    if self.front >= self.back
    {
      return None;
    }

    let index = self.front;
    self.front += 1;

    match index
    {
      0 =>
      {
        // SAFETY: `front` and `back` never cross (guarded above), so this field is
        // reborrowed at most once across the whole iteration — either here, from the
        // front, or in `next_back`, from the back, but never both — so this can never
        // alias a mutable reference already handed out by a previous call.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.0) ) }
      },
      1 =>
      {
        // SAFETY: see the arm above.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.1) ) }
      },
      2 =>
      {
        // SAFETY: see the arm above.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.2) ) }
      },
      _ => unreachable!(),
    }
  }

  fn size_hint( &self ) -> ( usize, Option< usize > )
  {
    let remaining = self.back - self.front;
    ( remaining, Some( remaining ) )
  }
}

impl< E > ExactSizeIterator for Tuple3IterMut< '_, E > {}

impl< E > DoubleEndedIterator for Tuple3IterMut< '_, E >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    if self.front >= self.back {
      return None;
    }

    self.back -= 1;

    match self.back {
      0 => {
        // SAFETY: see `next` — `front`/`back` never cross, so each field is reborrowed
        // at most once across the whole iteration.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.0) ) }
      },
      1 => {
        // SAFETY: see the arm above.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.1) ) }
      },
      2 => {
        // SAFETY: see the arm above.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.2) ) }
      },
      _ => unreachable!(),
    }
  }
}

impl< E: Clone > VectorIter< E, 3 > for ( E, E, E )
{
  #[ inline ]
  fn vector_iter< 'tuple_ref >( &'tuple_ref self ) -> impl VectorIteratorRef< 'tuple_ref, &'tuple_ref E >
  where
    E : 'tuple_ref,
  {
    Tuple3Iter
    {
      tuple : self,
      front : 0,
      back : 3,
    }
  }
}

impl< E: Clone > VectorIterMut< E, 3 > for ( E, E, E )
{
  #[ inline ]
  fn vector_iter_mut< 'tuple_ref >( &'tuple_ref mut self ) -> impl VectorIterator< 'tuple_ref, &'tuple_ref mut E >
  where
    E : 'tuple_ref,
  {
    Tuple3IterMut
    {
      tuple : self,
      front : 0,
      back : 3,
    }
  }
}

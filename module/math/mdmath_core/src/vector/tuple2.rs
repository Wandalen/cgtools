use core::mem::{ size_of, align_of };

use super::{Collection, ConstLength, IntoArray, ArrayRef, ArrayMut, VectorIter, VectorIteratorRef, VectorIterMut, VectorIterator};

// Fix(BUG-449): compile-time layout proof shared by `ArrayRef::array_ref`/`ArrayMut::vector_mut`
// below -- see `array_ref`'s own BUG-449 comment for the full root-cause/pitfall writeup.
const fn assert_tuple2_array_layout< E >()
{
  assert!( size_of::< ( E, E ) >() == size_of::< [ E ; 2 ] >(), "(E,E) and [E;2] must have the same size" );
  assert!( align_of::< ( E, E ) >() == align_of::< [ E ; 2 ] >(), "(E,E) and [E;2] must have the same alignment" );
  assert!( core::mem::offset_of!( ( E, E ), 0 ) == 0, "field 0 must be at byte offset 0" );
  assert!( core::mem::offset_of!( ( E, E ), 1 ) == size_of::< E >(), "field 1 must immediately follow field 0 with no padding" );
}

// = 2

impl< E > Collection for ( E, E )
{
  type Scalar = E;
}

impl< E > ConstLength for ( E, E )
{
  const LEN : usize = 2;
}

impl< E > IntoArray< E, 2 > for ( E, E )
{
  #[ inline ]
  fn into_array( self ) -> [ E ; 2 ]
  {
    [ self.0, self.1 ]
  }
}

impl< E > ArrayRef< E, 2 > for ( E, E )
{
  // Fix(BUG-449): replaced the runtime-only, debug-only `debug_assert_eq!` of *total* size/align
  // with an unconditional, compile-time proof (`assert_tuple2_array_layout`) that also checks
  // per-field byte offsets -- the exact property this cast depends on.
  // Root cause: the previous `debug_assert_eq!` only checked that `(E,E)` and `[E;2]` have the
  // same *total* size and alignment -- necessary but not sufficient to justify the cast (e.g. a
  // hypothetical layout with fields reordered or padded between them would still pass a
  // total-size/align check while making the cast produce a wrong result). The check also ran
  // only in debug builds, so a future rustc layout change would silently produce UB with zero
  // runtime signal in release.
  // Pitfall: proving *total* size/align equality is not the same as proving *field-order*
  // equality -- when justifying a same-layout cast, assert the exact property the cast depends
  // on (field 0 at byte offset 0, field 1 at byte offset `size_of::<E>()`), not merely a
  // necessary consequence of it. `core::mem::offset_of!` supports raw tuple types directly
  // (stable since Rust 1.77) and, combined with an inline `const { }` block (stable since Rust
  // 1.79), lets this check run unconditionally at compile time instead of conditionally at
  // runtime.
  #[ inline( always ) ]
  fn array_ref( &self ) -> &[ E ; 2 ]
  {
    const { assert_tuple2_array_layout::< E >() };

    // SAFETY: the compile-time check above proves, for this concrete `E`, that `(E,E)` and
    // `[E;2]` share total size, alignment, and per-field byte offsets -- the exact and complete
    // set of properties this reinterpret-cast depends on. The resulting reference's lifetime is
    // tied to `self`'s via the raw-pointer-cast-then-reborrow pattern, so it cannot outlive the
    // data it points to.
    #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
    unsafe { &*( std::ptr::from_ref::< ( E, E ) >( self ).cast::< [ E ; 2 ] >() ) }
  }
}

impl< E > ArrayMut< E, 2 > for ( E, E )
{
  // Fix(BUG-449): see `ArrayRef::array_ref`'s own BUG-449 comment above (same root cause and
  // fix, mirrored here for the mutable accessor).
  #[ inline( always ) ]
  fn vector_mut( &mut self ) -> &mut [ E ; 2 ]
  {
    const { assert_tuple2_array_layout::< E >() };

    // SAFETY: see `ArrayRef::array_ref` above -- same compile-time proof, mutable variant.
    #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
    unsafe { &mut *( std::ptr::from_mut::< ( E, E ) >( self ).cast::< [ E ; 2 ] >() ) }
  }
}

// Fix(BUG-122): `index : usize` was shared between `next()` and `next_back()`, whose match
// arms were hardcoded per-direction — after a `next()` call, `next_back()` reinterpreted the
// resulting `index` as if counted from the back, returning the same field `next()` already
// returned (as a second, harmless-but-wrong `&E`) while the other field was never yielded.
// Root cause: same shared-single-cursor shape as the already-fixed `Tuple2IterMut` (BUG-050),
// just never itself updated when that fix landed.
// Pitfall: a hand-rolled `DoubleEndedIterator` needs independent front/back cursors even when
// the yielded references are shared and aliasing-safe — a single shared counter is a
// correctness bug, not just a soundness one, under mixed `.next()`/`.next_back()` sequences.
#[ derive( Clone ) ]
struct Tuple2Iter< 'tuple_ref, E >
{
  tuple : &'tuple_ref ( E, E ),
  front : usize,
  back : usize,
}

impl< 'tuple_ref, E > Iterator for Tuple2Iter< 'tuple_ref, E >
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

    match index
    {
      0 => Some( &self.tuple.0 ),
      1 => Some( &self.tuple.1 ),
      _ => unreachable!(),
    }
  }

  fn size_hint( &self ) -> ( usize, Option< usize > )
  {
    let remaining = self.back - self.front;
    ( remaining, Some( remaining ) )
  }
}

impl< E > ExactSizeIterator for Tuple2Iter< '_, E > {}

impl< E > DoubleEndedIterator for Tuple2Iter< '_, E >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    if self.front >= self.back
    {
      return None;
    }

    self.back -= 1;

    match self.back
    {
      0 => Some( &self.tuple.0 ),
      1 => Some( &self.tuple.1 ),
      _ => unreachable!(),
    }
  }
}

// Fix(BUG-050): `index : usize` was shared between `next()` and `next_back()`, whose match
// arms were hardcoded per-direction — mixing the two calls on one iterator (e.g. `.next()`
// then `.next_back()`) double-yielded the same tuple field as two simultaneously-live `&mut E`
// references instead of two disjoint ones.
// Root cause: copy-pasted from the immutable `Tuple2Iter` above (where aliasing `&E` is
// harmless) into a `&mut` context without redesigning the cursor for unique-borrow safety.
// Pitfall: a hand-rolled `DoubleEndedIterator` yielding `&mut` references needs independent
// front/back cursors (mirrors `core::slice::IterMut`), never a single shared counter — always
// test a mixed `.next()`/`.next_back()` sequence, not just pure-forward or pure-`.rev()`.
struct Tuple2IterMut< 'tuple_ref, E >
{
  tuple : &'tuple_ref mut ( E, E ),
  front : usize,
  back : usize,
}

impl< 'tuple_ref, E > Iterator for Tuple2IterMut< 'tuple_ref, E >
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
      _ => unreachable!(),
    }
  }

  fn size_hint( &self ) -> ( usize, Option< usize > )
  {
    let remaining = self.back - self.front;
    ( remaining, Some( remaining ) )
  }
}

impl< E > ExactSizeIterator for Tuple2IterMut< '_, E > {}

impl< E > DoubleEndedIterator for Tuple2IterMut< '_, E >
{
  fn next_back( &mut self ) -> Option< Self::Item >
  {
    if self.front >= self.back
    {
      return None;
    }

    self.back -= 1;

    match self.back
    {
      0 =>
      {
        // SAFETY: see `next` — `front`/`back` never cross, so each field is reborrowed
        // at most once across the whole iteration.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.0) ) }
      },
      1 =>
      {
        // SAFETY: see the arm above.
        #[ expect( unsafe_code, reason = "unsafe is intentional in this vector core; every unsafe block carries a SAFETY comment enforced by undocumented_unsafe_blocks = deny" ) ]
        unsafe { Some( &mut *std::ptr::addr_of_mut!(self.tuple.1) ) }
      },
      _ => unreachable!(),
    }
  }
}

impl< E: Clone > VectorIter< E, 2 > for ( E, E )
{
  #[ inline ]
  fn vector_iter< 'tuple_ref >( &'tuple_ref self ) -> impl VectorIteratorRef< 'tuple_ref, &'tuple_ref E >
  where
    E : 'tuple_ref,
  {
    Tuple2Iter
    {
      tuple : self,
      front : 0,
      back : 2,
    }
  }
}

impl< E: Clone > VectorIterMut< E, 2 > for ( E, E )
{
  #[ inline ]
  fn vector_iter_mut< 'tuple_ref >( &'tuple_ref mut self ) -> impl VectorIterator< 'tuple_ref, &'tuple_ref mut E >
  where
    E : 'tuple_ref,
  {
    Tuple2IterMut
    {
      tuple : self,
      front : 0,
      back : 2,
    }
  }
}

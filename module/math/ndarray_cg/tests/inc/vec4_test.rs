use super::*;

use the_module::{ F32x4, I32x4 };

// test_kind: bug_reproducer(BUG-043)
/// ## Root Cause
/// `Vector<E,4>::w()` (`vector/vec4/general.rs`) was authored by copying `z()`'s body and updating
/// only the doc comment and method name — the internal index literal was never bumped from `2` to
/// `3`, so `w()` silently returned the 3rd component instead of the 4th for every element type.
/// ## Why Not Caught
/// No test in this crate exercised `.w()` on a bare `Vector<E,4>` at all; the only 3 `.w()` call
/// sites in the workspace go through `Quat<E>`, which has its own independent, correct `self.0[3]`
/// implementation and never delegates to this method.
/// ## Fix Applied
/// Changed `w()`'s body from `self.0[ 2 ]` to `self.0[ 3 ]` (`vector/vec4/general.rs`).
/// ## Prevention
/// For every constructed value with N pairwise-distinct components, assert
/// `component_accessor[ i ]( v ) == v[ i ]` for all `i` in `0..N` — catches a copy-pasted accessor
/// whose index was never updated to match its new name.
/// ## Pitfall
/// Before the fix this assertion fails with `w() == z()` (both equal the 3rd component) for any
/// vector whose 3rd and 4th components differ.
#[ test ]
fn accessor_test()
{
  let v = I32x4::new( 1, 2, 3, 4 );
  assert_eq!( v.x(), 1 );
  assert_eq!( v.y(), 2 );
  assert_eq!( v.z(), 3 );
  assert_eq!( v.w(), 4 );
  assert_ne!( v.w(), v.z() );

  let v = F32x4::new( 1.0, 2.0, 3.0, 4.0 );
  // `v` is constructed from the same literals compared against — no arithmetic occurs, so
  // the stored components are bit-identical to the literals.
  #[ expect( clippy::float_cmp, reason = "assertions check exact expected values; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]
  {
    assert_eq!( v.x(), 1.0 );
    assert_eq!( v.y(), 2.0 );
    assert_eq!( v.z(), 3.0 );
    assert_eq!( v.w(), 4.0 );
    assert_ne!( v.w(), v.z() );
  }
}

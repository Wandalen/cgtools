use super::the_module;
use the_module::{ Mesh, Skeleton };
use std::rc::Rc;
use std::cell::RefCell;

/// ## Root Cause
/// `Mesh`'s manual `Clone` impl ( `src/webgl/mesh.rs` ) built the clone's `skeleton` field by
/// cloning the *value* behind `self.skeleton`'s `Rc< RefCell< Skeleton > > `, writing that fresh
/// clone back into `self`'s own `RefCell`, and then returning `self`'s original `Rc` -- unlike
/// `primitives` two lines above, which correctly wraps a fresh `Rc::new( RefCell::new( .. ) )`
/// around the cloned value. The clone never got its own `Rc`: both the original and the "cloned"
/// `Mesh` ended up sharing the identical `Rc< RefCell< Skeleton > > ` allocation.
/// ## Why Not Caught
/// `Mesh` had zero test coverage of any kind prior to this bug -- no existing test constructed a
/// `Mesh`, let alone cloned one with a `skeleton` set.
/// ## Fix Applied
/// The `skeleton` arm now mirrors `primitives`'s own pattern exactly:
/// `self.skeleton.as_ref().map( | s | Rc::new( RefCell::new( s.borrow().clone() ) ) )` -- a fresh
/// `Rc` wrapping a clone of the pointee, matching `Primitive::clone`'s identical
/// geometry/material pattern elsewhere in this same crate.
/// ## Prevention
/// Any `Clone` arm for an `Rc< RefCell< T > > ` field must produce a *new* `Rc`, never `self`'s
/// own via a bare `.clone()` on the `Rc` itself -- compare a new arm against an already-correct
/// sibling field in the same `impl` before trusting it compiles, since both shapes type-check
/// identically.
/// ## Pitfall
/// `Rc::strong_count`/`Rc::ptr_eq` are the only way to catch this from the outside -- the cloned
/// `Mesh`'s `skeleton` field is `Some( .. )` either way, so a naive `is_some()` assertion passes
/// on both the buggy and the fixed code; the defect is only visible by comparing identity, not
/// presence.
#[ test ]
fn clone_gives_the_clone_its_own_independent_skeleton()
{
  let original_skeleton = Rc::new( RefCell::new( Skeleton::default() ) );
  let mut mesh = Mesh::new();
  mesh.skeleton = Some( original_skeleton.clone() );

  assert_eq!( Rc::strong_count( &original_skeleton ), 2, "sanity check: only `original_skeleton` and `mesh.skeleton` should hold this allocation before cloning" );

  let cloned_mesh = mesh.clone();
  let cloned_skeleton = cloned_mesh.skeleton.expect( "a Mesh cloned from one with Some( skeleton ) must also carry Some( skeleton )" );

  assert!( !Rc::ptr_eq( &original_skeleton, &cloned_skeleton ), "Mesh::clone must give the clone its own independent Skeleton Rc, not alias the original's -- animating one instance's skeleton must not silently animate the other's" );
  assert_eq!( Rc::strong_count( &original_skeleton ), 2, "cloning a Mesh must not bump the original Skeleton Rc's strong count -- the clone must hold a separate allocation, not a third reference to this one" );
}

mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use mingl::geometry::BoundingBox;
  use crate::webgl::Primitive;
  use crate::webgl::Skeleton;

  /// Represents a collection of renderable primitives.
  #[ derive( Debug, Default ) ]
  pub struct Mesh
  {
    /// A vector holding the primitives that constitute the mesh. Each primitive is shared and mutable.
    pub primitives : Vec< Rc< RefCell< Primitive > > >,
    /// Stores matrices for every [`Node`](crate::webgl::Node) for skinning [`Mesh`]
    pub skeleton : Option< Rc< RefCell< Skeleton > > >,
    /// Whether this node casts shadows
    pub is_shadow_caster : bool,
  }

  impl Clone for Mesh
  {
    fn clone( &self ) -> Self
    {
      Self
      {
        primitives :
        {
          self.primitives.iter()
          .map( | p | Rc::new( RefCell::new( p.borrow().clone() ) ) )
          .collect::< Vec< _ > >()
        },
        // Fix(BUG-256): `s.clone()` cloned the `Rc` pointer itself, so the "cloned" `Mesh` ended
        // up aliasing the exact same `Rc< RefCell< Skeleton > > ` as `self` -- animating or
        // re-posing one instance's skeleton silently animated the other's too, breaking
        // `Node::tree_clone`'s documented "new independent scene graph subtree" contract. The
        // `s.borrow().clone()` / `*s.borrow_mut() = clone` dance in between accomplished nothing
        // observable for the clone (it only overwrote `self`'s own `Skeleton` with a fresh,
        // identical-by-value clone of itself, as a side effect of calling `.clone()` on `self`)
        // while masking that the `Rc` itself was never duplicated.
        // Root cause: unlike `primitives` immediately above (which correctly wraps a fresh
        // `Rc::new( RefCell::new( .. ) )` around the cloned value), this arm cloned the
        // `Skeleton` *value* but discarded the fresh clone back into `self`'s own `RefCell`
        // instead of a new `Rc`, then returned `self`'s original `Rc` for the new `Mesh` too.
        // Pitfall: `Rc<RefCell<T>>::clone()` type-checks identically whether it duplicates the
        // pointer (aliasing) or the pointee (independent) -- the two only diverge at the value
        // level, which no compiler warning catches; always compare a new `Clone` arm against an
        // existing correct one in the same `impl` (here, `primitives`) rather than trusting it
        // compiles.
        skeleton : self.skeleton.as_ref()
        .map( | s | Rc::new( RefCell::new( s.borrow().clone() ) ) ),
        is_shadow_caster : self.is_shadow_caster,
      }
    }
  }

  impl Mesh
  {
    /// Creates a new, empty `Mesh`.
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Adds a primitive to the mesh.
    ///
    /// * `primitive`: The primitive to be added.
    pub fn primitive_add( &mut self, primitive : Rc< RefCell< Primitive > > )
    {
      self.primitives.push( primitive );
    }

    /// Calculates and returns the combined bounding box for all primitives in the scene.
    #[ must_use ]
    pub fn bounding_box( &self ) -> BoundingBox
    {
      let mut bbox = BoundingBox::default();

      for primitive in &self.primitives
      {
        bbox.combine_mut( &primitive.borrow().bounding_box() );
      }

      bbox
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Mesh
  };
}

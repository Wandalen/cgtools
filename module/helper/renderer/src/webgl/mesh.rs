mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use mingl::geometry::BoundingBox;
  use crate::webgl::Primitive;
  use crate::webgl::Skeleton;

  /// Represents a collection of renderable primitives.
  #[ derive( Default ) ]
  pub struct Mesh
  {
    /// A vector holding the primitives that constitute the mesh. Each primitive is shared and mutable.
    pub primitives : Vec< Rc< RefCell< Primitive > > >,
    /// Stores matrices for every [`Node`] for skinning [`Mesh`]
    pub skeleton : Option< Rc< RefCell< Skeleton > > >
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
        skeleton : self.skeleton.as_ref()
        .map
        (
          | s |
          {
            let clone = s.borrow().clone();
            *s.borrow_mut() = clone;
            s.clone()
          }
        )
      }
    }
  }

  impl Mesh
  {
    /// Creates a new, empty `Mesh`.
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Adds a primitive to the mesh.
    ///
    /// * `primitive`: The primitive to be added.
    pub fn add_primitive( &mut self, primitive : Rc< RefCell< Primitive > > )
    {
      self.primitives.push( primitive );
    }

    /// Calculates and returns the combined bounding box for all primitives in the scene.
    pub fn bounding_box( &self ) -> BoundingBox
    {
      let mut bbox = BoundingBox::default();

      for primitive in self.primitives.iter()
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

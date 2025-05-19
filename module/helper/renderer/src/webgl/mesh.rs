mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use crate::webgl::Primitive;

  /// Represents a collection of renderable primitives.
  #[ derive( Default ) ]
  pub struct Mesh
  {
    /// A vector holding the primitives that constitute the mesh. Each primitive is shared and mutable.
    pub primitives : Vec< Rc< RefCell< Primitive > > >,
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
  }
}

crate::mod_interface!
{
  orphan use
  {
    Mesh
  };
}

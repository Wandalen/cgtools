mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use crate::webgl::Primitive;

  #[ derive( Default ) ]
  pub struct Mesh
  {
    pub primitives : Vec< Rc< RefCell< Primitive > > >,
  }

  impl Mesh
  {
    pub fn new() -> Self
    {
      Self::default()
    }

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

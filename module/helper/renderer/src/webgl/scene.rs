mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use minwebgl as gl;
  use crate::webgl::Node;

  #[ derive( Default ) ]
  pub struct Scene
  { 
    pub children : Vec< Rc< RefCell< Node > > >,
  }

  impl Scene
  {
    pub fn new() -> Self
    {
      Self::default()
    }

    pub fn add( &mut self, node : Rc< RefCell< Node > > )
    {
      self.children.push( node );
    }

    pub fn traverse< F >( &self, callback : &mut F ) -> Result< (), gl::WebglError >
    where F : FnMut( Rc< RefCell< Node > > ) -> Result< (), gl::WebglError >
    {
      for node in self.children.iter()
      {
        ( *callback )( node.clone() )?;
        node.borrow().traverse( callback )?;
      }

      Ok( () )
    }

    pub fn update_world_matrix( &mut self )
    {
      let mut identity = gl::F32x4x4::default();
      *identity.scalar_mut( gl::Ix2( 0, 0 ) ) = 1.0;
      *identity.scalar_mut( gl::Ix2( 1, 1 ) ) = 1.0;
      *identity.scalar_mut( gl::Ix2( 2, 2 ) ) = 1.0;
      *identity.scalar_mut( gl::Ix2( 3, 3 ) ) = 1.0;

      for child in self.children.iter_mut()
      {
        child.borrow_mut().update_world_matrix( identity );
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Scene
  };
}
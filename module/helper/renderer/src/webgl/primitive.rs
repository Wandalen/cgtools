mod private
{
  use std::{ cell::RefCell, collections::HashMap, rc::Rc };
  use minwebgl as gl;
  use crate::webgl::{ Geometry, Material };

  pub struct Primitive
  {
    pub geometry : Rc< RefCell< Geometry > >,
    pub material : Rc< RefCell< Material > >
  }

  impl Primitive
  {
    pub fn apply
    ( 
      &self,
      gl : &gl::WebGl2RenderingContext,
      locations : &HashMap< String, Option< gl::WebGlUniformLocation > > 
    ) -> Result< (), gl::WebglError >
    {
      self.material.borrow().apply( gl, locations )?;
      self.geometry.borrow().apply( gl )?;

      Ok( () )
    }

    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.material.borrow().bind( gl );
      self.geometry.borrow().bind( gl );
    }

    pub fn draw( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.geometry.borrow().draw( gl );
    }

    pub fn center( &self ) -> gl::F32x3
    {
      self.geometry.borrow().center()
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Primitive
  };
}
mod private
{
  use std::{ cell::RefCell, collections::HashMap, rc::Rc };
  use minwebgl as gl;
  use crate::webgl::{ Geometry, Material };

  /// Represents a renderable object composed of geometry and material.
  pub struct Primitive
  {
    /// The geometry of the primitive.
    pub geometry : Rc< RefCell< Geometry > >,
    /// The material of the primitive.
    pub material : Rc< RefCell< Material > >
  }

  impl Primitive
  {
    /// Uploads the material properties and geometry data to the GPU.
    ///
    /// * `gl`: The `WebGl2RenderingContext` to use for uploading.
    /// * `locations`: A hash map of uniform locations in the shader program.
    pub fn upload
    ( 
      &self,
      gl : &gl::WebGl2RenderingContext,
      locations : &HashMap< String, Option< gl::WebGlUniformLocation > > 
    ) -> Result< (), gl::WebglError >
    {
      self.material.borrow().upload( gl, locations )?;
      self.geometry.borrow().upload( gl )?;

      Ok( () )
    }

    /// Binds the material and geometry for rendering.
    ///
    /// * `gl`: The `WebGl2RenderingContext` to use for binding.
    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.material.borrow().bind( gl );
      self.geometry.borrow().bind( gl );
    }

    /// Draws the primitive using the currently bound material and geometry.
    ///
    /// * `gl`: The `WebGl2RenderingContext` to use for drawing.
    pub fn draw( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.geometry.borrow().draw( gl );
    }

    /// Returns the center point of the primitive's geometry.
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
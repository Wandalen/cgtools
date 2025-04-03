use std::collections::HashMap;

use minwebgl as gl;

use crate::{buffer::Buffer, primitive::Primitive};

pub struct Mesh
{
  primitives : Vec< Primitive >
}

impl Mesh 
{
  pub fn new
  ( 
    gl : &gl::WebGl2RenderingContext,
    mesh : &gltf::Mesh,
    buffers : &HashMap< usize, Buffer >
  ) -> Result< Self, gl::WebglError >
  {

    let mut primitives = Vec::new();
    for p in mesh.primitives()
    {
      let primitive = Primitive::new( gl, &p, buffers )?;
      primitives.push( primitive );
    }

    Ok
    ( 
      Self
      {
        primitives
      }
    )
  }

  pub fn render( &self, gl : &gl::WebGl2RenderingContext )
  {
    for p in self.primitives.iter()
    {
      p.render( gl );
    }
  }
}

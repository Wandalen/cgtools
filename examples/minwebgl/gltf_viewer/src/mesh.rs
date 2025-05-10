use std::{cell::RefCell, collections::HashMap, rc::Rc};

use minwebgl as gl;

use crate::{buffer::Buffer, node::Node, primitive::Primitive};

pub struct Mesh
{
  pub primitives : Vec< Rc< Primitive > >,
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
      let primitive = Rc::new( Primitive::new( gl, &p, buffers )? );
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

  pub fn apply( &self, gl : &gl::WebGl2RenderingContext )
  {
    for p in self.primitives.iter()
    {
      p.apply( gl );
    }
  }
}

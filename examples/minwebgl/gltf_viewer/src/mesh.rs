use std::{cell::RefCell, collections::HashMap, rc::Rc};

use minwebgl as gl;

use crate::{buffer::Buffer, node::Node, primitive::Primitive};

pub struct Mesh
{
  pub primitives : Vec< Primitive >,
  pub parent_node : Rc< RefCell< Node > >
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
    let parent_node = Rc::default();
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
        primitives,
        parent_node
      }
    )
  }

  pub fn set_parent( &mut self, node : Rc< RefCell< Node > > )
  {
    self.parent_node = node;
  }

  pub fn apply( &self, gl : &gl::WebGl2RenderingContext )
  {
    for p in self.primitives.iter()
    {
      p.apply( gl );
    }
  }
}

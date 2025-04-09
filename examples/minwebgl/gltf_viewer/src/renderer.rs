use std::{cell::RefCell, rc::Rc};

use mingl::CameraOrbitControls;
use minwebgl as gl;

use crate::{node::Node, scene::Scene};


pub struct Renderer< 'a >
{
  nodes : &'a [ Rc< RefCell< Node > > ]
}

impl< 'a > Renderer< 'a > 
{
  pub fn new( nodes : &'a [ Rc< RefCell< Node > > ] ) -> Self
  {

    Self
    {
      nodes
    }
  }    


  pub fn render
  ( 
    &self, 
    gl : &gl::WebGl2RenderingContext,
    scene : Scene, 
    camera : &CameraOrbitControls 
  )
  {

  }
}
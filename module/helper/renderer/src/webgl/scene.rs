use std::{cell::RefCell, rc::Rc};

use minwebgl as gl;

use crate::node::Node;

pub struct Scene
{ 
  pub children : Vec< Rc< RefCell< Node > > >,
}

impl Scene
{
  pub fn new( scene : &gltf::Scene, nodes : &[ Rc< RefCell< Node > > ] ) -> Self
  {
    let mut children = Vec::new();
    for gltf_node in scene.nodes()
    {
      let node = nodes[ gltf_node.index() ].clone();
      node.borrow_mut().add_children( &gltf_node, nodes );
      children.push( node );
    }

    Self
    {
      children
    }
  }

  pub fn traverse< F >( &self, callback : &mut F )
  where F : FnMut( Rc< RefCell< Node > > )
  {
    for node in self.children.iter()
    {
      ( *callback )( node.clone() );
      node.borrow().traverse( callback );
    }
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
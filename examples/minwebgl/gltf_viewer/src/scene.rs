use std::rc::Rc;

use minwebgl as gl;

use crate::node::Node;

pub struct Scene
{ 
  children : Vec< Node >,
}

impl Scene
{
  pub fn new( scene : &gltf::Scene ) -> Self
  {
    let mut children = Vec::new();
    for node in scene.nodes()
    {
      children.push( Node::new( &node ) );
    }

    Self
    {
      children
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
      child.update_world_matrix( identity );
    }
  }

  pub fn render( &self )
  {

  }
}
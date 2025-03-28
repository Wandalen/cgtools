use std::rc::Rc;

use minwebgl as gl;

pub struct Scene
{ 
  // An array of indices into the node_list
  children : Vec< u32 >,
  node_list : Rc< Vec< Node > >
}

impl Scene
{
  //
  pub fn render( &self )
  {
    for id in self.children
    { 
      // Could pass an identity matrix as parent's transform
      self.node_list[ id ].render() //
    }
  }
}
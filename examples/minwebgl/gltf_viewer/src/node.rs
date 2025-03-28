use std::rc::Rc;

use minwebgl as gl;

pub struct Node
{
  parent : u32,
	children : Vec< u32 >,
	local_matrix : gl::F32x4x4,
	material : gl::WebGLProgram,
	node_list : Rc< Vec< Node > >
}

impl Node
{
	// Should get the matrix information from the parent
	// Should either pass the cumultive transformation matrix to the render function
	// or add another function that will update the world transformation for the Node
	pub fn render( &self )
	{

	}
}
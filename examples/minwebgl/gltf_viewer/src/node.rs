use std::{cell::RefCell, rc::Rc};

use minwebgl::{ self as gl };

use crate::{mesh::Mesh, program::ProgramInfo};

pub enum Object3D
{
  Mesh( usize ),
  Other
}

impl Default for Object3D 
{
  fn default() -> Self 
  {
    Self::Mesh( 0 )    
  }    
}

#[ derive( Default ) ]
pub struct Node
{
  children : Vec< Rc< RefCell< Node > > >,
  pub object : Object3D,
  matrix : gl::F32x4x4,
  world_matrix : gl::F32x4x4,
  scale : gl::F32x3,
  translation : gl::F32x3,
  rotation : glam::Quat
}

impl Node 
{
  pub fn new( node : &gltf::Node ) -> Self
  {
    let mut result = Self::default();

    // An object this node referes to
    result.object = if let Some( mesh ) = node.mesh()
    {
      Object3D::Mesh( mesh.index() )
    }
    else
    {
      Object3D::Other
    };

    match node.transform()
    {
      gltf::scene::Transform::Matrix { matrix } =>
      {
        result.matrix = gl::F32x4x4::from_column_major( glam::Mat4::from_cols_array_2d( &matrix ).to_cols_array() );
        let mat = glam::Mat4::from_cols_array_2d( &matrix );
        let ( s, r, t ) = mat.to_scale_rotation_translation();

        result.scale = s.to_array().into();
        result.translation = t.to_array().into();
        result.rotation = r;
      },
      gltf::scene::Transform::Decomposed { translation, rotation, scale } =>
      {
        result.scale = scale.into();
        result.translation = translation.into();
        result.rotation = glam::Quat::from_array( rotation );

        let mat = glam::Mat4::from_scale_rotation_translation( scale.into(), glam::Quat::from_array( rotation ), translation.into() );
        result.matrix = gl::F32x4x4::from_column_major( mat.to_cols_array() );
      }
    }

    result
  }

  pub fn update_world_matrix( &mut self, parent_mat : gl::F32x4x4 ) 
  {
    self.world_matrix = parent_mat * self.matrix;

    for child in self.children.iter_mut()
    {
      child.borrow_mut().update_world_matrix( self.world_matrix );
    }
  }

  pub fn add_child( &mut self, child : Rc< RefCell< Node > > )
  {
    self.children.push( child );
  }

  pub fn add_children( &mut self, gltf_node : &gltf::Node, nodes : &[ Rc< RefCell< Node > > ] )
  {
    //let mut text = String::new();
    //text.push_str( &format!( "Node: {}\n", gltf_node.index()) );
    for c in gltf_node.children()
    {
      //text.push_str( &format!( "\tChild: {}\n", c.index()) );
      let node = nodes[ c.index() ].clone();
      node.borrow_mut().add_children( &c, nodes );
      self.add_child( node );
    }

   // gl::info!( "NODE INFO:\n{}", text );

    
  }

  pub fn apply
  ( 
    &self, 
    gl : &gl::WebGl2RenderingContext,
    program_info : &ProgramInfo 
  )
  {
    let locations = program_info.get_locations();

    gl::uniform::matrix_upload
    ( 
      &gl, 
      locations.get( "worldMatrix" ).unwrap().clone(),
      self.world_matrix.to_array().as_slice(), 
      true 
    ).unwrap();
  }
}
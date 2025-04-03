use std::{cell::RefCell, rc::Rc};

use minwebgl::{ self as gl };

use crate::mesh::Mesh;

pub enum Object3D
{
  Mesh( usize ),
  Camera( usize ),
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
  id : usize,
  children : Vec< Node >,
  object : Object3D,
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
      Object3D::Camera( node.camera().unwrap().index() )
    };

    // Childrens of the node
    result.children = Vec::new();
    for cn in node.children()
    {
      result.children.push( Node::new( &cn ) );
    }

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
      child.update_world_matrix( self.world_matrix );
    }
  }

  pub fn render( &self, gl : &gl::WebGl2RenderingContext, meshes : &[ Mesh ] )
  {
    match self.object
    {
      Object3D::Camera( id ) => { gl::info!( "Trying to render a camera" ) },
      Object3D::Mesh( id ) => { meshes[ id ].render( gl ); }
    }
  }
}
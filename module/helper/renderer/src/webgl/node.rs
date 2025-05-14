mod private
{
  use std::{ cell::RefCell, collections::HashMap, rc::Rc };
  use minwebgl::{ self as gl };
  use crate::webgl::Mesh;

  pub enum Object3D
  {
    Mesh( Rc< RefCell< Mesh > > ),
    Other
  }

  impl Default for Object3D
  {
    fn default() -> Self
    {
      Self::Other
    }
  }

  #[ derive( Default ) ]
  pub struct Node
  {
    pub children : Vec< Rc< RefCell< Node > > >,
    pub object : Object3D,
    // Local matrix of the node
    pub matrix : gl::F32x4x4,
    // Global matrix of the node( including all of its parents )
    pub world_matrix : gl::F32x4x4,
    pub scale : gl::F32x3,
    pub translation : gl::F32x3,
    pub rotation : glam::Quat,
    needs_local_matrix_update : bool
  }

  impl Node
  {
    pub fn new() -> Self
    {
      Self::default()
    }

    pub fn set_scale( &mut self, scale : impl Into< gl::F32x3 > )
    {
      self.scale = scale.into();
      self.needs_local_matrix_update = true;
    }

    pub fn get_scale( &self ) -> gl::F32x3
    {
      self.scale
    }

    pub fn set_translation( &mut self, translation : impl Into< gl::F32x3 > )
    {
      self.translation = translation.into();
      self.needs_local_matrix_update = true;
    }

    pub fn get_translation( &self ) -> gl::F32x3
    {
      self.translation
    }

    pub fn set_rotation( &mut self, rotation : glam::Quat )
    {
      self.rotation = rotation;
      self.needs_local_matrix_update = true;
    }

    pub fn get_rotation( &self ) -> glam::Quat
    {
      self.rotation
    }

    pub fn update_local_matrix( &mut self )
    {
      let mat = glam::Mat4::from_scale_rotation_translation
      ( 
        self.scale.to_array().into(), 
        self.rotation, 
        self.translation.to_array().into() 
      );
      self.matrix = gl::F32x4x4::from_column_major( mat.to_cols_array() );
      self.needs_local_matrix_update = false;
    }

    pub fn update_world_matrix( &mut self, parent_mat : gl::F32x4x4 )
    {
      if self.needs_local_matrix_update
      {
        self.update_local_matrix();
      }

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

    pub fn upload
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      locations : &HashMap< String, Option< gl::WebGlUniformLocation > >
    )
    {
      gl::uniform::matrix_upload
      (
        &gl,
        locations.get( "worldMatrix" ).unwrap().clone(),
        self.world_matrix.to_array().as_slice(),
        true
      ).unwrap();
    }

    pub fn traverse< F >( &self, callback : &mut F ) -> Result< (), gl::WebglError >
    where F : FnMut( Rc< RefCell< Node > > ) -> Result< (), gl::WebglError >
    {
      for node in self.children.iter()
      {
        ( *callback )( node.clone() )?;
        node.borrow().traverse( callback )?;
      }

      Ok( () )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Node,
    Object3D
  };
}
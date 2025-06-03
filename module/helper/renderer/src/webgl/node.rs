mod private
{
  use std::{ cell::RefCell, collections::HashMap, rc::Rc };
  use minwebgl::{ self as gl };
  use mingl::{ geometry::BoundingBox, F32x3 };
  use crate::webgl::Mesh;

  /// Represents a 3D object that can be part of the scene graph.  
  pub enum Object3D
  {
    /// A mesh object, containing geometry and material information.
    Mesh( Rc< RefCell< Mesh > > ),
    /// A placeholder for other types of 3D objects.
    Other
  }

  impl Default for Object3D
  {
    fn default() -> Self
    {
      Self::Other
    }
  }

  /// Represents a node in the scene graph. Each node can have children, an associated 3D object, and transformations.
  #[ derive( Default ) ]
  pub struct Node
  {
    /// The child nodes of this node.
    pub children : Vec< Rc< RefCell< Node > > >,
    /// The 3D object associated with this node.
    pub object : Object3D,
    /// The local transformation matrix of the node.
    pub matrix : gl::F32x4x4,
    /// The global transformation matrix of the node, including the transformations of its parents.
    pub world_matrix : gl::F32x4x4,
    /// The local scale of the node.
    pub scale : gl::F32x3,
    /// The local translation of the node.
    pub translation : gl::F32x3,
    /// The local rotation of the node as a quaternion.
    pub rotation : glam::Quat,
    /// A flag indicating whether the local matrix needs to be updated based on scale, translation, or rotation changes.
    needs_local_matrix_update : bool
  }

  impl Node
  {
    /// Creates a new `Node` with default values.
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Sets the local scale of the node.
    ///
    /// * `scale`: The new scale as a type that can be converted into `gl::F32x3`.
    pub fn set_scale( &mut self, scale : impl Into< gl::F32x3 > )
    {
      self.scale = scale.into();
      self.needs_local_matrix_update = true;
    }

    /// Returns the current local scale of the node.
    pub fn get_scale( &self ) -> gl::F32x3
    {
      self.scale
    }

    /// Sets the local translation of the node.
    ///
    /// * `translation`: The new translation as a type that can be converted into `gl::F32x3`.
    pub fn set_translation( &mut self, translation : impl Into< gl::F32x3 > )
    {
      self.translation = translation.into();
      self.needs_local_matrix_update = true;
    }

    /// Returns the current local translation of the node.
    pub fn get_translation( &self ) -> gl::F32x3
    {
      self.translation
    }

    /// Sets the local rotation of the node.
    ///
    /// * `rotation`: The new rotation as a `glam::Quat`.
    pub fn set_rotation( &mut self, rotation : glam::Quat )
    {
      self.rotation = rotation;
      self.needs_local_matrix_update = true;
    }

    /// Returns the current local rotation of the node.
    pub fn get_rotation( &self ) -> glam::Quat
    {
      self.rotation
    }

    /// Updates the local transformation matrix based on the current scale, rotation, and translation.
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

    /// Updates the world transformation matrix of the node and recursively updates the world matrices of its children.
    ///
    /// * `parent_mat`: The world matrix of the parent node. For the root node, this should be the identity matrix.
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

    /// Adds a child node to this node.
    ///
    /// * `child`: The child node to be added.
    pub fn add_child( &mut self, child : Rc< RefCell< Node > > )
    {
      self.children.push( child );
    }

    /// Uploads the world transformation matrix of this node to the GPU as a uniform.
    ///
    /// * `gl`: The `WebGl2RenderingContext`.
    /// * `locations`: A hash map of uniform locations in the shader program.
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

    /// Traverses the node and its descendants, calling the provided callback function for each node.
    ///
    /// * `callback`: A mutable closure or function that takes an `Rc<RefCell<Node>>` as input and returns a `Result<(), gl::WebglError>`.
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

    pub fn bounding_box( &self ) -> BoundingBox
    {
      let mut bbox = BoundingBox::default();

      match self.object
      {
        Object3D::Mesh( ref mesh ) => 
        { 
          bbox = mesh.borrow().bounding_box().apply_transform( self.matrix );
        },
        _ => {}
      }

      for child in self.children.iter()
      {
        bbox.combine_mut( &child.borrow().bounding_box().apply_transform( self.matrix ) );
      }

      bbox
    }

    pub fn center( &self ) -> F32x3
    {
      self.bounding_box().center()
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
mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use mingl::{ geometry::BoundingBox };
  use minwebgl as gl;
  use crate::webgl::Node;

  /// Represents a scene containing a hierarchy of nodes.
  pub struct Scene
  { 
    /// The root-level children of the scene.
    pub children : Vec< Rc< RefCell< Node > > >,
    /// Name of the scene
    name : Option< Box< str > >,
    /// The local transformation matrix of the scene.
    matrix : gl::F32x4x4,
    /// The local scale of the scene.
    scale : gl::F32x3,
    /// The local translation of the scene.
    translation : gl::F32x3,
    /// The local rotation of the scene as a quaternion.
    rotation : gl::QuatF32,
    /// A flag indicating whether the local matrix needs to be updated based on scale, translation, or rotation changes.
    needs_local_matrix_update : bool,
    /// A flag indicating whether the world matrix of the scene's children needs to be updated
    needs_update_child_world_matrix : bool,
    /// The bounding box of the node's object in world space.
    bounding_box : BoundingBox
  }

  impl Default for Scene
  {
    fn default() -> Self 
    {
      let identity_matrix = gl::math::mat4x4::identity();

      Scene
      {
        name : None,
        children : Vec::new(),
        matrix : identity_matrix,
        scale : gl::F32x3::splat( 1.0 ),
        translation : gl::F32x3::default(),
        rotation : gl::Quat::default(),
        needs_local_matrix_update : false,
        needs_update_child_world_matrix : false,
        bounding_box : BoundingBox::default()
      }
    }
  }

  impl Clone for Scene
  {
    fn clone( &self ) -> Self
    {
      let mut children = Vec::with_capacity( self.children.len() );
      
      for child in &self.children 
      {
        children.push( child.borrow().clone_tree() );
      }

      Self
      {
        children,
        name : self.name.clone(),
        matrix : self.matrix,
        scale : self.scale,
        translation : self.translation,
        rotation : self.rotation,
        needs_local_matrix_update : self.needs_local_matrix_update,
        needs_update_child_world_matrix : self.needs_update_child_world_matrix,
        bounding_box : self.bounding_box
      }
    }
  }

  impl Scene
  {
    /// Creates a new, empty `Scene`.
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Sets the name of the scene.
    pub fn set_name( &mut self, name : impl Into< Box< str > > )
    {
      self.name = Some( name.into() );
    }

    /// Returns an owned clone of the scene's name.
    pub fn get_name( &self ) -> Option< Box< str > >
    {
      self.name.clone()
    }

    /// Sets the local scale of the scene.
    ///
    /// * `scale`: The new scale as a type that can be converted into `gl::F32x3`.
    pub fn set_scale( &mut self, scale : impl Into< gl::F32x3 > )
    {
      self.scale = scale.into();
      self.needs_local_matrix_update = true;
    }

    /// Returns the current local scale of the scene.
    pub fn get_scale( &self ) -> gl::F32x3
    {
      self.scale
    }

    /// Sets the local translation of the scene.
    ///
    /// * `translation`: The new translation as a type that can be converted into `gl::F32x3`.
    pub fn set_translation( &mut self, translation : impl Into< gl::F32x3 > )
    {
      self.translation = translation.into();
      self.needs_local_matrix_update = true;
    }

    /// Returns the current local translation of the scene.
    pub fn get_translation( &self ) -> gl::F32x3
    {
      self.translation
    }

    /// Sets the local rotation of the scene.
    ///
    /// * `rotation`: The new rotation as a `gl::QuatF32`.
    pub fn set_rotation( &mut self, rotation : gl::QuatF32 )
    {
      self.rotation = rotation;
      self.needs_local_matrix_update = true;
    }

    /// Returns the current local rotation of the scene.
    pub fn get_rotation( &self ) -> gl::QuatF32
    {
      self.rotation
    }

    /// Returns a slice of the scene's children.
    pub fn get_children( &self ) -> &[ Rc< RefCell< Node > > ]
    {
      self.children.as_slice()
    }

    /// Sets the local transformation matrix for the node.
    pub fn set_local_matrix( &mut self, matrix : gl::F32x4x4 )
    {
      let Some( ( translation, rotation, scale ) ) = matrix.decompose()
      else
      {
        return;
      };

      self.set_translation( translation );
      self.set_rotation( rotation );
      self.set_scale( scale );

      self.matrix = matrix;
      self.needs_local_matrix_update = false;
      self.needs_update_child_world_matrix = true;
    }

    /// Returns the current local transformation matrix.
    pub fn get_local_matrix( &self ) -> gl::F32x4x4
    {
      self.matrix
    }

    /// Updates the local transformation matrix based on the current scale, rotation, and translation.
    pub fn update_local_matrix( &mut self )
    {
      let mat = gl::F32x4x4::from_scale_rotation_translation
      (
        self.scale,
        self.rotation,
        self.translation
      );
      self.matrix = mat;
      self.needs_local_matrix_update = false;
      self.needs_update_child_world_matrix = true;
    }

    /// Removes a child node at the given index.
    pub fn remove_child( &mut self, id : usize ) -> Rc< RefCell< Node > >
    {
      self.children.remove( id )
    }

    /// Adds a node to the scene's root level.
    ///
    /// * `node`: The node to be added.
    pub fn add( &mut self, node : Rc< RefCell< Node > > )
    {
      self.children.push( node );
    }

    /// Traverses all nodes in the scene, starting from the root children, and calls the provided callback function for each node.
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

    /// Updates the world transformation matrices of all nodes in the scene, starting from the root.
    pub fn update_world_matrix( &mut self )
    {
      if self.needs_local_matrix_update
      {
        self.update_local_matrix();
      }

      // Recursively update the world matrix of each root node and its descendants.
      for child in self.children.iter_mut()
      {
        child.borrow_mut().update_world_matrix( self.matrix, self.needs_update_child_world_matrix );
      }
    }

    /// Computes the bounding box for the entire hierarchy of a node.
    ///
    /// This function calculates a `BoundingBox` that encompasses the current node and all of its descendants.
    pub fn bounding_box( &self ) -> BoundingBox
    {
      let mut bbox = BoundingBox::default();

      for child in self.children.iter()
      {
        bbox.combine_mut( &child.borrow().bounding_box_hierarchical() );
      }

      bbox
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Scene
  };
}
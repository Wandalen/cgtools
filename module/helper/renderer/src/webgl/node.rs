mod private
{
  use std::{ cell::RefCell, collections::HashMap, rc::Rc };
  use minwebgl as gl;
  use mingl::{ geometry::BoundingBox, F32x3, F32x4x4 };
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
    /// The name of the node.
    name : Option< Box< str > >,
    /// The parent node of this node
    parent : Option< Rc< RefCell< Node > > >,
    /// The child nodes of this node.
    children : Vec< Rc< RefCell< Node > > >,
    /// The 3D objects associated with this node.
    pub object : Object3D,
    /// The local transformation matrix of the node.
    matrix : gl::F32x4x4,
    /// The global transformation matrix of the node, including the transformations of its parents.
    world_matrix : gl::F32x4x4,
    /// The normal matrix for transforming normals, derived from the world matrix.
    normal_matrix : gl::F32x3x3,
    /// The local scale of the node.
    scale : gl::F32x3,
    /// The local translation of the node.
    translation : gl::F32x3,
    /// The local rotation of the node as a quaternion.
    rotation : gl::QuatF32,
    /// A flag indicating whether the local matrix needs to be updated based on scale, translation, or rotation changes.
    needs_local_matrix_update : bool,
    /// A flag indicating whether the world matrix needs to be updated.
    needs_world_matrix_update : bool,
    /// The bounding box of the node's object in world space.
    bounding_box : BoundingBox
  }

  impl Node
  {
    /// Creates a new `Node` with default values.
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Clones the node and all of its descendants, creating a new independent scene graph subtree.
    pub fn clone_tree( &self ) -> Rc< RefCell< Self > >
    {
      let object = match &self.object
      {
        Object3D::Mesh( mesh ) =>
        {
          Object3D::Mesh( Rc::new( RefCell::new( mesh.borrow().clone() ) ) )
        },
        Object3D::Other => Object3D::Other
      };

      let clone = Self
      {
        name : self.name.clone(),
        parent : None,
        children : vec![],
        object,
        matrix : self.matrix,
        world_matrix : self.world_matrix,
        normal_matrix : self.normal_matrix,
        scale : self.scale,
        translation : self.translation,
        rotation : self.rotation,
        needs_local_matrix_update : self.needs_local_matrix_update,
        needs_world_matrix_update : self.needs_world_matrix_update,
        bounding_box : self.bounding_box
      };

      let clone_rc = Rc::new( RefCell::new( clone ) );

      self.children.iter()
      .for_each
      (
        | n |
        {
          let child = n.borrow().clone_tree();
          child.borrow_mut().set_parent( Some( clone_rc.clone() ) );
          clone_rc.borrow_mut().add_child( child.clone() );
        }
      );

      clone_rc
    }

    /// Sets the name of the node.
    pub fn set_name( &mut self, name : impl Into< Box< str > > )
    {
      self.name = Some( name.into() );
    }

    /// Returns an owned clone of the node's name.
    pub fn get_name( &self ) -> Option< Box< str > >
    {
      self.name.clone()
    }

    /// Returns a slice of the node's children.
    pub fn get_children( &self ) -> &[ Rc< RefCell< Node > > ]
    {
      self.children.as_slice()
    }

    /// Sets the parent of the node.
    pub fn set_parent( &mut self, parent : Option< Rc< RefCell< Node > > > )
    {
      self.parent = parent;
    }

    /// Returns a reference to the node's parent.
    pub fn get_parent( &self ) -> &Option< Rc< RefCell< Node > > >
    {
      &self.parent
    }

    /// Removes a child node at the given index.
    pub fn remove_child( &mut self, id : usize ) -> Rc< RefCell< Node > >
    {
      self.children.remove( id )
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
    /// * `rotation`: The new rotation as a `gl::QuatF32`.
    pub fn set_rotation( &mut self, rotation : gl::QuatF32 )
    {
      self.rotation = rotation;
      self.needs_local_matrix_update = true;
    }

    /// Returns the current local rotation of the node.
    pub fn get_rotation( &self ) -> gl::QuatF32
    {
      self.rotation
    }

    /// Sets the local transformation matrix for the node.
    pub fn set_local_matrix( &mut self, matrix : F32x4x4 )
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
    }

    /// Sets the world transformation matrix for the node.
    pub fn set_world_matrix( &mut self, matrix : F32x4x4 )
    {
      self.world_matrix = matrix;
      self.normal_matrix = matrix.truncate().inverse().unwrap().transpose();
      self.compute_bounding_box();
      self.needs_world_matrix_update = false;
    }

    /// Returns the current world transformation matrix.
    pub fn get_world_matrix( &self ) -> F32x4x4
    {
      self.world_matrix
    }

    /// Returns the current local transformation matrix.
    pub fn get_local_matrix( &self ) -> F32x4x4
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
      self.matrix = gl::F32x4x4::from_column_major( mat.to_array());
      self.needs_local_matrix_update = false;
      self.needs_world_matrix_update = true;
    }

    /// Updates the world transformation matrix of the node and its children.
    ///
    /// * `parent_mat`: The world matrix of the parent node. For the root node, this should be the identity matrix.
    /// * `needs_world_matrix_update`: A flag to force an update, even if the node's flags are false.
    pub fn update_world_matrix( &mut self, parent_mat : gl::F32x4x4, mut needs_world_matrix_update : bool )
    {
      if self.needs_local_matrix_update
      {
        self.update_local_matrix();
      }

      if needs_world_matrix_update || self.needs_world_matrix_update
      {
        self.set_world_matrix( parent_mat * self.matrix );
        needs_world_matrix_update = true;
      }

      for child in self.children.iter_mut()
      {
        child.borrow_mut().update_world_matrix( self.world_matrix, needs_world_matrix_update );
      }
    }

    /// Adds a child node to this node.
    ///
    /// * `child`: The child node to be added.
    pub fn add_child( &mut self, child : Rc< RefCell< Node > > )
    {
      self.children.push( child );
    }

    /// Inserts a child node at a specific index.
    pub fn insert_child( &mut self, id : usize, child : Rc< RefCell< Node > > )
    {
      if id >= self.children.len()
      {
        self.add_child( child );
      }
      else
      {
        self.children.insert( id, child );
      }
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
      if let Object3D::Mesh( mesh ) = &self.object
      {
        if let Some( skeleton ) = &mesh.borrow().skeleton
        {
          skeleton.borrow_mut().upload( gl, locations );
        }
      }

      gl::uniform::matrix_upload
      (
        &gl,
        locations.get( "worldMatrix" ).unwrap().clone(),
        self.world_matrix.to_array().as_slice(),
        true
      ).unwrap();

      gl::uniform::matrix_upload
      (
        &gl,
        locations.get( "normalMatrix" ).unwrap().clone(),
        self.normal_matrix.to_array().as_slice(),
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

    /// Returns the pre-computed bounding box of the node.
    pub fn bounding_box( &self ) -> BoundingBox
    {
      self.bounding_box
    }

    /// Computes the bounding box for the current node based on its `Object3D` type.
    pub fn compute_bounding_box( &mut self )
    {
      match self.object
      {
        Object3D::Mesh( ref mesh ) =>
        {
          self.bounding_box = mesh.borrow().bounding_box().apply_transform( self.world_matrix );
        },
        _ => {}
      }
    }

    /// Computes the hierarchical bounding box for the node and all of its children.
    ///
    /// This function starts with the node's own bounding box and then recursively
    /// combines the hierarchical bounding boxes of all its children. This creates a
    /// single bounding box that encapsulates the entire sub-tree.
    pub fn bounding_box_hierarchical( &self ) -> BoundingBox
    {
      let mut bbox = self.bounding_box;

      for child in self.children.iter()
      {
        bbox.combine_mut( &child.borrow().bounding_box_hierarchical() );
      }

      bbox
    }

    /// Returns the center point of the node's pre-computed bounding box.
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

mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use mingl::geometry::BoundingBox;
  use minwebgl as gl;
  use crate::webgl::Node;

  /// Represents a scene containing a hierarchy of nodes.
  #[ derive( Default ) ]
  pub struct Scene
  { 
    /// The root-level children of the scene.
    pub children : Vec< Rc< RefCell< Node > > >,
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
        children
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
      // Initialize an identity matrix for the root nodes' parent world matrix.
      let mut identity = gl::F32x4x4::default();
      *identity.scalar_mut( gl::Ix2( 0, 0 ) ) = 1.0;
      *identity.scalar_mut( gl::Ix2( 1, 1 ) ) = 1.0;
      *identity.scalar_mut( gl::Ix2( 2, 2 ) ) = 1.0;
      *identity.scalar_mut( gl::Ix2( 3, 3 ) ) = 1.0;

      // Recursively update the world matrix of each root node and its descendants.
      for child in self.children.iter_mut()
      {
        child.borrow_mut().update_world_matrix( identity, false );
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
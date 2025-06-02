mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use mingl::geometry::BoundingBox;
use minwebgl as gl;
  use crate::webgl::{Node, Object3D};

  /// Represents a scene containing a hierarchy of nodes.
  #[ derive( Default ) ]
  pub struct Scene
  { 
    /// The root-level children of the scene.
    pub children : Vec< Rc< RefCell< Node > > >,
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
        child.borrow_mut().update_world_matrix( identity );
      }
    }

    pub fn bounding_box( &self ) -> BoundingBox
    {
      let mut bbox = BoundingBox::default();

      let mut calc_bounding_box = 
      | 
        node : Rc< RefCell< Node > > 
      | -> Result< (), gl::WebglError >
      {
        let node = node.borrow();
        if let Object3D::Mesh( ref mesh ) = node.object
        {
          let mbbox = mesh.borrow().bounding_box();

          let tmin = node.world_matrix * gl::F32x4::from( [ mbbox.min.x(), mbbox.min.y(), mbbox.min.z(), 1.0 ] );
          let tmax = node.world_matrix * gl::F32x4::from( [ mbbox.max.x(), mbbox.max.y(), mbbox.max.z(), 1.0 ] );

          bbox.min = bbox.min.min( gl::F32x3::from( [ tmin.x(), tmin.y(), tmin.z() ] ) );
          bbox.max = bbox.max.max( gl::F32x3::from( [ tmax.x(), tmax.y(), tmax.z() ] ) );
        }

        Ok( () )
      };

      self.traverse( &mut calc_bounding_box ).unwrap();

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
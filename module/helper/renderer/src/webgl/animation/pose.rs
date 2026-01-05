mod private
{
  use rustc_hash::FxHashMap;
  use std::
  {
    cell::RefCell,
    rc::Rc
  };
  use minwebgl as gl;
  use gl::{ F64x3, F32x3, QuatF32, QuatF64 };
  use crate::webgl::Node;
  use crate::webgl::animation::Transform;

  /// Use this struct for saving simple 3D transformations
  /// for every [`Node`] of one object
  pub struct Pose
  {
    /// Stores [`Transform`] for every [`Node`]
    transforms : FxHashMap< Box< str >, Transform >,
    /// Stores links to [`Node`]'s
    nodes : FxHashMap< Box< str >, Rc< RefCell< Node > > >
  }

  impl Pose
  {
    /// [`Pose`] constructor
    ///
    /// Parameters:
    /// * _nodes - list of [`Node`]'s which current 3D
    ///   transformation parameters are used for defining [`Pose`]
    pub fn new( _nodes : &[ Rc< RefCell< Node > > ] ) -> Self
    {
      let transforms = _nodes.iter()
      .filter_map
      (
        | n |
        {
          let Some( name ) = n.borrow().get_name()
          else
          {
            return None;
          };

          let transform = Transform
          {
            translation : F64x3::from_array( n.borrow().get_translation().map( | v | v as f64 ) ),
            rotation : QuatF64::from( n.borrow().get_rotation().0.map( | v | v as f64 ) ),
            scale : F64x3::from_array( n.borrow().get_scale().map( | v | v as f64 ) )
          };

          Some( ( name, transform ) )
        }
      )
      .collect::< FxHashMap< _, _ > >();

      let nodes = _nodes.iter()
      .filter_map
      (
        | n |
        {
          let Some( name ) = n.borrow().get_name()
          else
          {
            return None;
          };

          Some( ( name, n.clone() ) )
        }
      )
      .collect::< FxHashMap< _, _ > >();

      Self
      {
        transforms,
        nodes
      }
    }

    /// Set [`Transform`]'s for each related [`Node`]
    pub fn set( &self )
    {
      for ( name, t ) in &self.transforms
      {
        if let Some( node ) = self.nodes.get( name )
        {
          let mut node_mut = node.borrow_mut();

          node_mut.set_translation( F32x3::from_array( t.translation.0.map( | v | v as f32 ) ) );
          node_mut.set_rotation( QuatF32::from( t.rotation.0.map( | v | v as f32 ) ) );
          node_mut.set_scale( F32x3::from_array( t.scale.0.map( | v | v as f32 ) ) );
        }
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Pose
  };
}

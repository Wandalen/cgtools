mod private
{
  use std::
  {
    cell::RefCell,
    collections::HashMap,
    rc::Rc
  };
  use minwebgl as gl;
  use gl::{ F64x3, F32x3, QuatF32, QuatF64 };
  use crate::webgl::
  {
    animation::base::
    {
      AnimatableComposition,
      MORPH_TARGET_PREFIX,
      ROTATION_PREFIX,
      SCALE_PREFIX,
      TRANSLATION_PREFIX
    },
    Node,
    Object3D
  };

  /// Skeletal animation property variants
  #[ derive( Clone ) ]
  pub enum AnimationProperty
  {
    /// Translation property
    Translation( F64x3 ),
    /// Rotation property
    Rotation( QuatF64 ),
    /// Scale property
    Scale( F64x3 ),
    /// Weight property
    Weights( Vec< f64 > )
  }

  /// Use this struct for saving simple 3D transformations
  /// for every [`Node`] of one object
  pub struct Pose
  {
    /// Stores [`AnimationProperty`]'ies for every [`Node`]. Represents state of [`Pose`]
    animatables : HashMap< Box< str >, AnimationProperty >,
    /// Stores links to [`Node`]'s
    nodes : HashMap< Box< str >, Rc< RefCell< Node > > >
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
      let animatables = _nodes.iter()
      .filter_map
      (
        | n |
        {
          let Some( name ) = n.borrow().get_name()
          else
          {
            return None;
          };

          let mut node_animatables: Vec< ( Box< str >, AnimationProperty ) > = vec!
          [
            (
              format!( "{}{}", name, TRANSLATION_PREFIX ).into_boxed_str(),
              AnimationProperty::Translation( F64x3::from_array( n.borrow().get_translation().map( | v | v as f64 ) ) )
            ),
            (
              format!( "{}{}", name, ROTATION_PREFIX ).into_boxed_str(),
              AnimationProperty::Rotation( QuatF64::from( n.borrow().get_rotation().0.map( | v | v as f64 ) ) )
            ),
            (
              format!( "{}{}", name, SCALE_PREFIX ).into_boxed_str(),
              AnimationProperty::Scale( F64x3::from_array( n.borrow().get_scale().map( | v | v as f64 ) ) )
            ),
          ];

          if let Object3D::Mesh( mesh ) = &n.borrow().object
          {
            if let Some( skeleton ) = &mesh.borrow().skeleton
            {
              if skeleton.borrow().has_morph_targets()
              {
                node_animatables.push
                (
                  (
                    format!( "{}{}", name, MORPH_TARGET_PREFIX ).into_boxed_str(),
                    AnimationProperty::Weights
                    (
                      skeleton.borrow().displacements_as_ref().as_ref().unwrap()
                      .get_morph_weights().borrow().iter().map( | v | *v as f64 )
                      .collect::< Vec< _ > >()
                    )
                  )
                );
              }
            }
          }

          Some( node_animatables )
        }
      )
      .flatten()
      .collect::< HashMap< Box< str >, AnimationProperty > >();

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
      .collect::< HashMap< _, _ > >();

      Self
      {
        animatables,
        nodes
      }
    }

    /// Get [`HashMap`] of related [`Node`]'s
    pub fn nodes_get( &self ) -> &HashMap< Box< str >, Rc< RefCell< Node > > >
    {
      &self.nodes
    }

    /// Get [`HashMap`] of related animated properties
    pub fn state_get( &self ) -> &HashMap< Box< str >, AnimationProperty >
    {
      &self.animatables
    }
  }

  impl AnimatableComposition for Pose
  {
    fn update( &mut self, _delta_time : f64 )
    {

    }

    fn as_any( &self ) -> &dyn core::any::Any
    {
      self
    }

    fn as_any_mut( &mut self ) -> &mut dyn core::any::Any
    {
      self
    }

    fn set( &self, nodes : &HashMap< Box< str >, Rc< RefCell< Node > > > )
    {
      for ( name, node ) in nodes
      {
        if let Some( AnimationProperty::Translation( translation ) ) = self.animatables.get
        (
          format!( "{}{}", name, TRANSLATION_PREFIX ).as_str()
        )
        {
          let translation = translation.0.map( | v | v as f32 );
          node.borrow_mut().set_translation( F32x3::from_array( translation ) );
        }

        if let Some( AnimationProperty::Rotation( rotation ) ) = self.animatables.get
        (
          format!( "{}{}", name, ROTATION_PREFIX ).as_str()
        )
        {
          let rotation = rotation.0.map( | v | v as f32 );
          node.borrow_mut().set_rotation( QuatF32::from( rotation ) );
        }

        if let Some( AnimationProperty::Scale( scale ) ) = self.animatables.get
        (
          format!( "{}{}", name, SCALE_PREFIX ).as_str()
        )
        {
          let scale = scale.0.map( | v | v as f32 );
          node.borrow_mut().set_scale( F32x3::from_array( scale ) );
        }

        if let Some( AnimationProperty::Weights( weights ) ) = self.animatables.get
        (
          format!( "{}{}", name, MORPH_TARGET_PREFIX ).as_str()
        )
        {
          let weights = weights.iter()
          .map( | v | *v as f32 )
          .collect::< Vec< _ > >();
          if let crate::webgl::Object3D::Mesh( mesh ) = &node.borrow().object
          {
            if let Some( skeleton ) = &mesh.borrow().skeleton
            {
              if let Some( displacements ) = skeleton.borrow().displacements_as_ref()
              {
                let weights_rc = displacements.get_morph_weights();
                let mut weights_mut = weights_rc.borrow_mut();
                for i in 0..weights.len().min( weights_mut.len() )
                {
                  weights_mut[ i ] = weights[ i ];
                }
              }
            }
          }
        }
      }
    }
  }

  impl Clone for Pose
  {
    fn clone( &self ) -> Self
    {
      Self
      {
        animatables : self.animatables.clone(),
        nodes : self.nodes.clone()
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Pose,
    AnimationProperty
  };
}

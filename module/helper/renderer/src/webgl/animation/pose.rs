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
    animatables : FxHashMap< Box< str >, AnimationProperty >,
    /// Stores links to [`Node`]'s
    nodes : FxHashMap< Box< str >, Rc< RefCell< Node > > >
  }

  impl Pose
  {
    /// [`Pose`] constructor
    ///
    /// Parameters:
    /// * nodes - list of [`Node`]'s which current 3D
    ///   transformation parameters are used for defining [`Pose`]
    ///
    /// # Panics
    ///
    /// Panics if a morph-target node's skeleton has no displacement data.
    pub fn new( nodes : &[ Rc< RefCell< Node > > ] ) -> Self
    {
      let animatables = nodes.iter()
      .filter_map
      (
        | n |
        {
          let name = n.borrow().name_get()?;

          let mut node_animatables: Vec< ( Box< str >, AnimationProperty ) > = vec!
          [
            (
              format!( "{name}{TRANSLATION_PREFIX}" ).into_boxed_str(),
              AnimationProperty::Translation( F64x3::from_array( n.borrow().translation_get().map( f64::from ) ) )
            ),
            (
              format!( "{name}{ROTATION_PREFIX}" ).into_boxed_str(),
              AnimationProperty::Rotation( QuatF64::from( n.borrow().rotation_get().0.map( f64::from ) ) )
            ),
            (
              format!( "{name}{SCALE_PREFIX}" ).into_boxed_str(),
              AnimationProperty::Scale( F64x3::from_array( n.borrow().scale_get().map( f64::from ) ) )
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
                    format!( "{name}{MORPH_TARGET_PREFIX}" ).into_boxed_str(),
                    AnimationProperty::Weights
                    (
                      skeleton.borrow().displacements_as_ref().as_ref().unwrap()
                      .morph_weights_get().borrow().iter().map( | v | f64::from(*v) )
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
      .collect::< FxHashMap< Box< str >, AnimationProperty > >();

      let nodes = nodes.iter()
      .filter_map
      (
        | n |
        {
          let name = n.borrow().name_get()?;

          Some( ( name, n.clone() ) )
        }
      )
      .collect::< FxHashMap< _, _ > >();

      Self
      {
        animatables,
        nodes
      }
    }

    /// Get [`FxHashMap`] of related [`Node`]'s
    #[ must_use ]
    pub fn nodes_get( &self ) -> &FxHashMap< Box< str >, Rc< RefCell< Node > > >
    {
      &self.nodes
    }

    /// Get [`FxHashMap`] of related animated properties
    #[ must_use ]
    pub fn state_get( &self ) -> &FxHashMap< Box< str >, AnimationProperty >
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

    fn set( &self, nodes : &FxHashMap< Box< str >, Rc< RefCell< Node > > > )
    {
      for ( name, node ) in nodes
      {
        if let Some( AnimationProperty::Translation( translation ) ) = self.animatables.get
        (
          format!( "{name}{TRANSLATION_PREFIX}" ).as_str()
        )
        {
          let translation = translation.0.map( | v | v as f32 );
          node.borrow_mut().translation_set( F32x3::from_array( translation ) );
        }

        if let Some( AnimationProperty::Rotation( rotation ) ) = self.animatables.get
        (
          format!( "{name}{ROTATION_PREFIX}" ).as_str()
        )
        {
          let rotation = rotation.0.map( | v | v as f32 );
          node.borrow_mut().rotation_set( QuatF32::from( rotation ) );
        }

        if let Some( AnimationProperty::Scale( scale ) ) = self.animatables.get
        (
          format!( "{name}{SCALE_PREFIX}" ).as_str()
        )
        {
          let scale = scale.0.map( | v | v as f32 );
          node.borrow_mut().scale_set( F32x3::from_array( scale ) );
        }

        if let Some( AnimationProperty::Weights( weights ) ) = self.animatables.get
        (
          format!( "{name}{MORPH_TARGET_PREFIX}" ).as_str()
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
                let weights_rc = displacements.morph_weights_get();
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

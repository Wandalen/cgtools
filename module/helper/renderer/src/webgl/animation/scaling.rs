mod private
{
  use rustc_hash::{ FxHashMap, FxHashSet };
  use std::{ rc::Rc, cell::RefCell };
  use animation::
  {
    Tween,
    Sequence,
    Sequencer,
    AnimatablePlayer
  };
  use mingl as gl;
  use gl::{ F32x3, F64x3, F64x4, QuatF32, QuatF64 };
  use crate::webgl::
  {
    Node,
    animation::
    {
      AnimatableComposition,
      base::
      {
        TRANSLATION_PREFIX,
        ROTATION_PREFIX,
        SCALE_PREFIX
      }
    }
  };

  /// Animation modifier that can scale animation for different groups of related [`Node`]'s
  #[ derive( Clone ) ]
  pub struct Scaler
  {
    /// Animation that must be scaled
    pub animation : Sequencer,
    /// Set of grouped [`Node`]'s with their scaling weights for
    /// each simple 3D transofrmation. Weights vector consist of
    /// such components:
    /// - x - transform
    /// - y - rotation
    /// - z - scale
    scaled_nodes : FxHashMap< Box< str >, ( Vec< Box< str > >, F64x4 ) >,
  }

  /// Converts a quaternion delta to axis-angle representation.
  ///
  /// # Arguments
  ///
  /// * `delta` - The quaternion representing the rotation difference
  ///
  /// # Returns
  ///
  /// A tuple of (axis, angle) where axis is a normalized F64x3 and angle is in radians
  fn quat_to_axis_angle( delta : QuatF64 ) -> ( F64x3, f64 )
  {
    let w = delta.0[ 3 ].clamp( -1.0, 1.0 );
    let angle = 2.0 * w.acos();
    let sin_half = ( 1.0 - w * w ).sqrt();

    let axis = if sin_half.abs() > std::f32::EPSILON as f64
    {
      F64x3::new
      (
        delta.0[ 0 ] / sin_half,
        delta.0[ 1 ] / sin_half,
        delta.0[ 2 ] / sin_half,
      )
    }
    else
    {
      F64x3::new( 1.0, 0.0, 0.0 )
    };

    ( axis, angle )
  }

  impl Scaler
  {
    /// Create new [`Scaler`]
    pub fn new( animation : Sequencer ) -> Self
    {
      Self
      {
        animation,
        scaled_nodes : FxHashMap::default()
      }
    }

    /// Add scaled nodes group
    pub fn add
    (
      &mut self,
      group_name : &str,
      node_names : Vec< Box< str > >,
      scale : F64x4
    )
    {
      self.scaled_nodes.insert( group_name.into(), ( node_names, scale ) );
    }

    /// Remove scaled nodes group
    pub fn remove( &mut self, group_name : Box< str > )
    {
      self.scaled_nodes.remove( &group_name );
    }

    /// Get reference to group nodes
    pub fn group_get( &self, group : &str ) -> Option< Vec< Box< str > > >
    {
      self.scaled_nodes.get( group.into() ).map( | ( n, _ ) | n ).cloned()
    }

    /// Get mutable reference to group nodes
    pub fn group_get_mut( &mut self, group : &str ) -> Option< &mut Vec< Box< str > > >
    {
      self.scaled_nodes.get_mut( group.into() ).map( | ( n, _ ) | n )
    }

    /// Get reference to group scale
    pub fn scale_get( &self, group : &str ) -> Option< &F64x4 >
    {
      self.scaled_nodes.get( group.into() ).map( | ( _, s ) | s )
    }

    /// Get mutable reference to group scale
    pub fn scale_get_mut( &mut self, group : &str ) -> Option< &mut F64x4 >
    {
      self.scaled_nodes.get_mut( group.into() ).map( | ( _, s ) | s )
    }

    /// Clear scaled_nodes
    pub fn clear( &mut self )
    {
      self.scaled_nodes.clear();
    }

    /// Applies scaled rotation to a node based on the animation and scaling factor.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to apply the rotation to
    /// * `name` - The name identifier for the node's rotation animation
    /// * `scale` - The scaling factor to apply to the rotation angle
    fn apply_scaled_rotation
    (
      &self,
      node : Rc< RefCell< Node > >,
      name : &str,
      scale : f64
    )
    {
      let Some( rotation ) = self.animation.get::< Sequence< Tween< QuatF64 > > >
      (
        &format!( "{}{}", name, ROTATION_PREFIX )
      )
      else
      {
        return;
      };

      let mut tweens = rotation.players_get();
      let current = rotation.current_id_get();

      for i in 0..( ( current + 1 ).min( tweens.len() ) )
      {
        if scale < 1.0 && i > 0
        {
          tweens[ i ].start_value = tweens[ i - 1 ].end_value;
        }

        let prev = tweens[ i ].start_value;
        let curr = tweens[ i ].end_value;
        let delta = prev.conjugate() * curr;

        let ( axis, angle ) = quat_to_axis_angle( delta );
        let angle_scaled = angle * scale;
        let delta_scaled = QuatF64::from_axis_angle( axis, angle_scaled );
        let new_end = prev * delta_scaled;
        tweens[ i ].end_value = new_end.normalize();
      }

      tweens[ 0 ].start_value = tweens.last().unwrap().end_value;

      let mut sequence = Sequence::new( tweens ).unwrap();
      sequence.update( rotation.time() );

      if let Some( tween ) = sequence.current_get()
      {
        let rotation = tween.value_get();
        let rotation = QuatF32::from( rotation.0.map( | v | v as f32 ) );
        node.borrow_mut().set_rotation( rotation );
      }
    }

    /// Applies unscaled transforms (translation, rotation, scale) to a node.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to apply transforms to
    /// * `name` - The name identifier for the node's animations
    fn apply_unscaled_transforms
    (
      &self,
      node : Rc< RefCell< Node > >,
      name : &str
    )
    {
      if let Some( translation ) = self.animation.get::< Sequence< Tween< F64x3 > > >
      (
        &format!( "{}{}", name, TRANSLATION_PREFIX )
      )
      {
        if let Some( translation ) = translation.current_get()
        {
          let translation = translation.value_get().0.map( | v | v as f32 );
          node.borrow_mut().set_translation( F32x3::from_array( translation ) );
        }
      }

      if let Some( rotation ) = self.animation.get::< Sequence< Tween< QuatF64 > > >
      (
        &format!( "{}{}", name, ROTATION_PREFIX )
      )
      {
        if let Some( rotation ) = rotation.current_get()
        {
          let rotation = rotation.value_get().0.map( | v | v as f32 );
          node.borrow_mut().set_rotation( QuatF32::from( rotation ) );
        }
      }

      if let Some( scale ) = self.animation.get::< Sequence< Tween< F64x3 > > >
      (
        &format!( "{}{}", name, SCALE_PREFIX )
      )
      {
        if let Some( scale ) = scale.current_get()
        {
          let scale = scale.value_get().0.map( | v | v as f32 );
          node.borrow_mut().set_scale( F32x3::from_array( scale ) );
        }
      }
    }
  }

  impl AnimatableComposition for Scaler
  {
    /// Updates all underlying [`animation::AnimatablePlayer`]'s
    fn update( &mut self, delta_time : f64 )
    {
      self.animation.update( delta_time );
    }

    /// Returns a type-erased reference to the underlying value
    fn as_any( &self ) -> &dyn core::any::Any
    {
      self
    }

    /// Returns a type-erased mutable reference to the underlying value
    fn as_any_mut( &mut self ) -> &mut dyn core::any::Any
    {
      self
    }

    /// Sets all simple 3D transformations for every
    /// [`Node`] related to this [`AnimatableComposition`]
    fn set( &self, nodes : &FxHashMap< Box< str >, Rc< RefCell< Node > > > )
    {
      let mut used_nodes = FxHashSet::< Box< str > >::default();

      // Apply scaled rotation to nodes in scaled groups
      for ( node_names, scales ) in self.scaled_nodes.values()
      {
        for name in node_names
        {
          let Some( node ) = nodes.get( name )
          else
          {
            continue;
          };

          used_nodes.insert( name.clone() );
          self.apply_scaled_rotation( node.clone(), name, scales.y() );
        }
      }

      // Apply unscaled transforms to remaining nodes
      for ( name, node ) in nodes
      {
        if !used_nodes.contains( name )
        {
          self.apply_unscaled_transforms( node.clone(), name );
        }
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Scaler
  };
}

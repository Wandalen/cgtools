mod private
{
  use std::collections::HashMap;
  use std::{ rc::Rc, cell::RefCell };
  use animation::
  {
    Tween,
    Sequence,
    Sequencer
  };
  use mingl as gl;
  use gl::{ F32x3, F64x3, QuatF32, QuatF64 };
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

  /// Precision for finding equal floats
  const EPSILON : f64 = 0.001;

  /// Normalize weights of blended animation values
  fn normalize_weights< T >( values : &mut [ ( T, f32 ) ] )
  {
    let sum = values.iter().map( | ( _, w ) | w ).sum::< f32 >();
    if sum > 0.0
    {
      let scale_factor = 1.0 / sum;
      values.iter_mut().for_each( | ( _, w ) | { *w *= scale_factor; } );
    }
  }

  /// Weighted animation blending implementation
  #[ derive( Clone ) ]
  pub struct Blender
  {
    /// Set of animations that must be blended using weights
    /// Weights vector consist of such components:
    /// - x - transform
    /// - y - rotation
    /// - z - scale
    weighted_animations : HashMap< Box< str >, ( Sequencer, F64x3 ) >,
    /// Flag that choose need normalize ( reduce to 1.0 ) sum of animation weights or not
    pub normalize : bool
  }

  impl Blender
  {
    /// Create new [`Blender`]
    pub fn new() -> Self
    {
      Self
      {
        weighted_animations : HashMap::new(),
        normalize : false
      }
    }

    /// Add weighted [`Sequence`]
    pub fn add
    (
      &mut self,
      name : Box< str >,
      animation : Sequencer,
      weights : F64x3
    )
    {
      self.weighted_animations.insert
      (
        name,
        ( animation, weights )
      );
    }

    /// Get mutable reference to weights of weighted [`Sequencer`] by name
    pub fn weights_get_mut( &mut self, name : Box< str > ) -> Option< &mut F64x3 >
    {
      self.weighted_animations.get_mut( &name )
      .map( | ( _, w ) | w )
    }

    /// Get weights of weighted [`Sequencer`] by name
    pub fn weights_get( &self, name : Box< str > ) -> Option< F64x3 >
    {
      self.weighted_animations.get( &name )
      .map( | ( _, w ) | w )
      .cloned()
    }

    /// Get reference to weighted [`Sequencer`] by name
    pub fn animation_get( &self, name : Box< str > ) -> Option< &Sequencer >
    {
      self.weighted_animations.get( &name )
      .map( | ( a, _ ) | a )
    }

    /// Get mutable weighted [`Sequencer`] by name
    pub fn animation_get_mut( &mut self, name : Box< str > ) -> Option< &mut Sequencer >
    {
      self.weighted_animations.get_mut( &name )
      .map( | ( a, _ ) | a )
    }

    /// Remove weighted [`Sequence`]
    pub fn remove( &mut self, name : Box< str > )
    {
      self.weighted_animations.remove( &name );
    }

    /// Check if blended animation is completed ( checks if all animations are completed )
    /// Better use before update
    pub fn is_completed( &self ) -> bool
    {
      let mut animations = self.weighted_animations.values()
      .map( | ( s, _ ) | s ).collect::< Vec< _ > >();

      animations.sort_by
      (
        | a, b |
        a.time().partial_cmp( &b.time() ).unwrap()
      );
      animations.reverse();

      let mut i = 1;
      while i < animations.len()
      {
        if ( animations[ i - 1 ].time() - animations[ i ].time() ).abs() > EPSILON
        {
          break;
        }
        i += 1;
      }

      if i == 1
      {
        animations[ 0 ].is_completed()
      }
      else
      {
        false
      }
    }

    /// Reset all blended animations
    pub fn reset( &mut self )
    {
      self.weighted_animations.values_mut()
      .for_each( | ( a, _ ) | a.reset() );
    }

    /// Blend translation values from all weighted animations for a specific node
    fn blend_translation( &self, name : &str, node : &Rc< RefCell< Node > > )
    {
      let mut values = vec![];

      for ( animation, weights ) in self.weighted_animations.values()
      {
        if let Some( translation ) = animation.get::< Sequence< Tween< F64x3 > > >
        (
          &format!( "{}{}", name, TRANSLATION_PREFIX )
        )
        {
          if let Some( translation ) = translation.current_get()
          {
            let weight = weights.x() as f32;
            values.push
            (
              (
                F32x3::from_array( translation.value_get().0.map( | v | v as f32 ) ),
                weight
              )
            );
          }
        }
      }

      if self.normalize
      {
        normalize_weights( &mut values );
      }

      let mut translation = F32x3::default();
      for ( t, w ) in values
      {
        translation += t * w;
      }
      node.borrow_mut().set_translation( translation );
    }

    /// Blend rotation values from all weighted animations for a specific node
    fn blend_rotation( &self, name : &str, node : &Rc< RefCell< Node > > )
    {
      let mut values = vec![];

      for ( animation, weights ) in self.weighted_animations.values()
      {
        if let Some( rotation ) = animation.get::< Sequence< Tween< QuatF64 > > >
        (
          &format!( "{}{}", name, ROTATION_PREFIX )
        )
        {
          if let Some( rotation ) = rotation.current_get()
          {
            let weight = weights.y() as f32;
            values.push
            (
              (
                QuatF32::from( rotation.value_get().0.map( | v | v as f32 ) ),
                weight
              )
            );
          }
        }
      }

      if self.normalize
      {
        normalize_weights( &mut values );
      }

      // NLERP
      let mut rotation = QuatF32::default();
      for ( r, w ) in values
      {
        rotation += r * w;
      }
      node.borrow_mut().set_rotation( rotation.normalize() );
    }

    /// Blend scale values from all weighted animations for a specific node
    fn blend_scale( &self, name : &str, node : &Rc< RefCell< Node > > )
    {
      let mut values = vec![];

      for ( animation, weights ) in self.weighted_animations.values()
      {
        if let Some( scale ) = animation.get::< Sequence< Tween< F64x3 > > >
        (
          &format!( "{}{}", name, SCALE_PREFIX )
        )
        {
          if let Some( scale ) = scale.current_get()
          {
            let weight = weights.z() as f32;
            values.push
            (
              (
                F32x3::from_array( scale.value_get().0.map( | v | v as f32 ) ),
                weight
              )
            );
          }
        }
      }

      if self.normalize
      {
        normalize_weights( &mut values );
      }

      let mut scale = F32x3::default();
      for ( s, w ) in values
      {
        scale += s * w;
      }
      node.borrow_mut().set_scale( scale );
    }
  }

  impl AnimatableComposition for Blender
  {
    /// Updates all underlying [`animation::AnimatablePlayer`]'s
    fn update( &mut self, delta_time : f64 )
    {
      for ( _, ( animation, _ ) ) in self.weighted_animations.iter_mut()
      {
        animation.update( delta_time );
        if animation.is_completed()
        {
          animation.reset();
        }
      }
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
    fn set( &self, nodes : &HashMap< Box< str >, Rc< RefCell< Node > > > )
    {
      for ( name, node ) in nodes
      {
        self.blend_translation( name, node );
        self.blend_rotation( name, node );
        self.blend_scale( name, node );
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Blender
  };
}

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

  /// Weighted animation blending implementation
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
    pub fn get_weights_mut( &mut self, name : Box< str > ) -> Option< &mut F64x3 >
    {
      self.weighted_animations.get_mut( &name )
      .map( | ( _, w ) | w )
    }

    /// Get weights of weighted [`Sequencer`] by name
    pub fn get_weights( &self, name : Box< str > ) -> Option< F64x3 >
    {
      self.weighted_animations.get( &name )
      .map( | ( _, w ) | w )
      .cloned()
    }

    /// Get reference to weighted [`Sequencer`] by name
    pub fn get_animation( &self, name : Box< str > ) -> Option< &Sequencer >
    {
      self.weighted_animations.get( &name )
      .map( | ( a, _ ) | a )
    }

    /// Get mutable weighted [`Sequencer`] by name
    pub fn get_animation_mut( &mut self, name : Box< str > ) -> Option< &mut Sequencer >
    {
      self.weighted_animations.get_mut( &name )
      .map( | ( a, _ ) | a )
    }

    /// Remove weighted [`Sequence`]
    pub fn remove( &mut self, name : Box< str > )
    {
      self.weighted_animations.remove( &name );
    }
  }

  impl AnimatableComposition for Blender
  {
    fn update( &mut self, delta_time : f64 )
    {
      for ( _, ( animation, _ ) ) in self.weighted_animations.iter_mut()
      {
        animation.update( delta_time );
      }
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
        let mut values = vec![];

        for ( animation, weights ) in self.weighted_animations.values()
        {
          if let Some( translation ) = animation.get::< Sequence< Tween< F64x3 > > >
          (
            &format!( "{}{}", name, TRANSLATION_PREFIX )
          )
          {
            if let Some( translation ) = translation.get_current()
            {
              let weight = weights.x() as f32;
              values.push
              (
                (
                  F32x3::from_array( translation.get_value().0.map( | v | v as f32 ) ),
                  weight as f32
                )
              );
            }
          }
        }

        if self.normalize
        {
          let scale_factor = 1.0 / values.iter().map( | ( _, w ) | w ).sum::< f32 >();
          values.iter_mut().for_each( | ( _, w ) | { *w *= scale_factor; } );
        }
        let mut translation = F32x3::default();
        for ( t, w ) in values
        {
          translation += t * w;
        }
        node.borrow_mut().set_translation( translation );

        let mut values = vec![];

        for ( animation, weights ) in self.weighted_animations.values()
        {
          if let Some( rotation ) = animation.get::< Sequence< Tween< QuatF64 > > >
          (
            &format!( "{}{}", name, ROTATION_PREFIX )
          )
          {
            if let Some( rotation ) = rotation.get_current()
            {
              let weight = weights.y() as f32;
              values.push
              (
                (
                  QuatF32::from(rotation.get_value().0.map( | v | v as f32 ) ),
                  weight
                )
              );
            }
          }
        }

        if self.normalize
        {
          let scale_factor = 1.0 / values.iter().map( | ( _, w ) | w ).sum::< f32 >();
          values.iter_mut().for_each( | ( _, w ) | { *w *= scale_factor; } );
        }
        // NLERP
        let mut rotation = QuatF32::default();
        for ( r, w ) in values
        {
          rotation += r * w;
        }
        node.borrow_mut().set_rotation( rotation.normalize() );

        let mut values = vec![];

        for ( animation, weights ) in self.weighted_animations.values()
        {
          if let Some( scale ) = animation.get::< Sequence< Tween< F64x3 > > >
          (
            &format!( "{}{}", name, SCALE_PREFIX )
          )
          {
            if let Some( scale ) = scale.get_current()
            {
              let weight = weights.z() as f32;
              values.push
              (
                (
                  F32x3::from_array( scale.get_value().0.map( | v | v as f32 ) ),
                  weight
                )
              );
            }
          }
        }

        if self.normalize
        {
          let scale_factor = 1.0 / values.iter().map( | ( _, w ) | w ).sum::< f32 >();
          values.iter_mut().for_each( | ( _, w ) | { *w *= scale_factor; } );
        }
        let mut scale = F32x3::default();
        for ( s, w ) in values
        {
          scale += s * w;
        }
        node.borrow_mut().set_scale( scale );
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

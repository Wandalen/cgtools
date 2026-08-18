mod private
{
  use rustc_hash::FxHashMap;
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

  /// Normalize weights of blended animation values
  pub fn weights_normalize< T >( values : &mut [ ( T, f32 ) ] )
  {
    let sum = values.iter().map( | ( _, w ) | w ).sum::< f32 >();
    if sum > 0.0
    {
      let scale_factor = 1.0 / sum;
      for ( _, w ) in values.iter_mut() { *w *= scale_factor; }
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
    weighted_animations : FxHashMap< Box< str >, ( Sequencer, F64x3 ) >,
    /// Flag that choose need normalize ( reduce to 1.0 ) sum of animation weights or not
    pub normalize : bool
  }

  impl Default for Blender
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl Blender
  {
    /// Create new [`Blender`]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        weighted_animations : FxHashMap::default(),
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
    pub fn weights_get_mut( &mut self, name : &str ) -> Option< &mut F64x3 >
    {
      self.weighted_animations.get_mut( name )
      .map( | ( _, w ) | w )
    }

    /// Get weights of weighted [`Sequencer`] by name
    #[ must_use ]
    pub fn weights_get( &self, name : &str ) -> Option< F64x3 >
    {
      self.weighted_animations.get( name )
      .map( | ( _, w ) | w )
      .copied()
    }

    /// Get reference to weighted [`Sequencer`] by name
    #[ must_use ]
    pub fn animation_get( &self, name : &str ) -> Option< &Sequencer >
    {
      self.weighted_animations.get( name )
      .map( | ( a, _ ) | a )
    }

    /// Get mutable weighted [`Sequencer`] by name
    pub fn animation_get_mut( &mut self, name : &str ) -> Option< &mut Sequencer >
    {
      self.weighted_animations.get_mut( name )
      .map( | ( a, _ ) | a )
    }

    /// Remove weighted [`Sequence`]
    pub fn remove( &mut self, name : &str )
    {
      self.weighted_animations.remove( name );
    }

    /// Check if blended animation is completed ( checks if all animations are completed )
    /// Better use before update
    // Fix(BUG-242): previously sorted animations by `.time()` and, when the top two were tied
    // ( within an EPSILON ), unconditionally returned `false` regardless of completion state;
    // when not tied, checked only the single animation with the largest raw elapsed time --
    // meaningless across animations of different durations -- and ignored every other one.
    // Neither branch implemented "all animations completed". Root cause: `.time()` measures
    // raw elapsed time, not completion; two animations can be time-tied while both genuinely
    // completed ( false negative ), or untied while the largest-time one alone is completed and
    // a shorter one isn't ( false positive ).
    #[ must_use ]
    pub fn is_completed( &self ) -> bool
    {
      !self.weighted_animations.is_empty()
      && self.weighted_animations.values().all( | ( s, _ ) | s.is_completed() )
    }

    /// Reset all blended animations
    pub fn reset( &mut self )
    {
      self.weighted_animations.values_mut()
      .for_each( | ( a, _ ) | a.reset() );
    }

    /// Blend translation values from all weighted animations for a specific node
    fn translation_blend( &self, name : &str, node : &Rc< RefCell< Node > > )
    {
      let mut values = vec![];

      for ( animation, weights ) in self.weighted_animations.values()
      {
        if let Some( translation ) = animation.get::< Sequence< Tween< F64x3 > > >
        (
          &format!( "{name}{TRANSLATION_PREFIX}" )
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

      // Fix(BUG-261): previously fell straight through to `translation_set` unconditionally,
      // even when no blended `Sequencer` had a translation channel for this node -- overwriting
      // the node's existing translation with `F32x3::default()` == `(0,0,0)`. glTF skeletal
      // rigs commonly omit a translation channel per-joint ( e.g. joints animated only via
      // rotation ), so this reachably zeroed out untouched joints' positions on every
      // `Blender::set` call.
      // Root cause: no `values.is_empty()` guard before applying the accumulated ( vacuous,
      // zero ) sum -- every sibling `AnimatableComposition` impl ( `Sequencer`, `Pose`,
      // `Scaler`, `Transition` ) instead skips the `_set()` call entirely when a channel is
      // absent, per the "skip-if-absent" convention established across this module.
      // Pitfall: an accumulator seeded from `Default::default()` is only safe to apply
      // unconditionally when "no contributions" and "identity contribution" are the same value
      // -- here they are not ( zero translation vs. "leave untouched" ), so emptiness must be
      // tracked and checked explicitly. See `rotation_blend`/`scale_blend` below for the same
      // fix applied to their own accumulators.
      if values.is_empty()
      {
        return;
      }

      if self.normalize
      {
        weights_normalize( &mut values );
      }

      let mut translation = F32x3::default();
      for ( t, w ) in values
      {
        translation += t * w;
      }
      node.borrow_mut().translation_set( translation );
    }

    /// Blend rotation values from all weighted animations for a specific node
    fn rotation_blend( &self, name : &str, node : &Rc< RefCell< Node > > )
    {
      let mut values = vec![];

      for ( animation, weights ) in self.weighted_animations.values()
      {
        if let Some( rotation ) = animation.get::< Sequence< Tween< QuatF64 > > >
        (
          &format!( "{name}{ROTATION_PREFIX}" )
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
        weights_normalize( &mut values );
      }

      // NLERP
      // Fix(BUG-183): a quaternion `q` and its negation `-q` represent the identical rotation,
      // but summing them naively does not -- if two blended clips' current rotations land in
      // opposite hemispheres ( dot product negative ), adding them cancels components instead of
      // blending, producing a wrong or near-zero result after normalize. Align each quaternion's
      // hemisphere to the running sum before accumulating it.
      //
      // Fix(BUG-196): the accumulator used to start from `QuatF32::default()`, which is the
      // IDENTITY quaternion `[0,0,0,1]` ( see `Quat::default()` ), not the additive zero
      // `[0,0,0,0]` a weighted-sum-then-normalize accumulator needs. Starting from identity
      // silently mixed an extra, unweighted "stay at identity" term into every blend -- even a
      // single fully-weighted clip no longer normalized back to its own rotation, it normalized
      // to a blend between its own rotation and identity. Seeding the accumulator from the first
      // entry itself ( scaled by its own weight ) sidesteps the question of what a "zero
      // rotation" would even mean here, and needs no hemisphere check of its own since there is
      // nothing to align against yet.
      // Fix(BUG-261): see `translation_blend` above for the shared root cause. This branch was
      // already explicit about the empty case, but explicitly wrong: it force-set the node's
      // rotation to `QuatF32::default()` ( identity, `[0,0,0,1]` ) whenever no blended
      // `Sequencer` had a rotation channel for this node, instead of leaving the node's
      // existing rotation untouched.
      let mut values_iter = values.into_iter();
      let Some( ( first_r, first_w ) ) = values_iter.next()
      else
      {
        return;
      };

      let mut rotation = first_r * first_w;
      for ( mut r, w ) in values_iter
      {
        if rotation.dot( &r ) < 0.0
        {
          r *= -1.0;
        }
        rotation += r * w;
      }
      node.borrow_mut().rotation_set( rotation.normalize() );
    }

    /// Blend scale values from all weighted animations for a specific node
    fn scale_blend( &self, name : &str, node : &Rc< RefCell< Node > > )
    {
      let mut values = vec![];

      for ( animation, weights ) in self.weighted_animations.values()
      {
        if let Some( scale ) = animation.get::< Sequence< Tween< F64x3 > > >
        (
          &format!( "{name}{SCALE_PREFIX}" )
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

      // Fix(BUG-261): see `translation_blend` above for the shared root cause -- same
      // unconditional fall-through into `scale_set`, applied to `F32x3::default()` == `(0,0,0)`
      // scale instead of the ( conventional, 1:1 ) "no scale channel present" outcome.
      if values.is_empty()
      {
        return;
      }

      if self.normalize
      {
        weights_normalize( &mut values );
      }

      let mut scale = F32x3::default();
      for ( s, w ) in values
      {
        scale += s * w;
      }
      node.borrow_mut().scale_set( scale );
    }
  }

  impl AnimatableComposition for Blender
  {
    /// Updates all underlying [`animation::AnimatablePlayer`]'s
    fn update( &mut self, delta_time : f64 )
    {
      for ( animation, _ ) in self.weighted_animations.values_mut()
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
    fn set( &self, nodes : &FxHashMap< Box< str >, Rc< RefCell< Node > > > )
    {
      for ( name, node ) in nodes
      {
        self.translation_blend( name, node );
        self.rotation_blend( name, node );
        self.scale_blend( name, node );
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    weights_normalize,
    Blender
  };
}

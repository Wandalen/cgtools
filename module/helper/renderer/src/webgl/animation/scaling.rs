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
    /// each simple 3D transformation. Weights vector consist of
    /// such components:
    /// - x - transform
    /// - y - rotation
    /// - z - scale
    /// - w - morph targets
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

    let axis = if sin_half.abs() > f64::from(f32::EPSILON)
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
    #[ must_use ]
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
    pub fn remove( &mut self, group_name : &str )
    {
      self.scaled_nodes.remove( group_name );
    }

    /// Get reference to group nodes
    #[ must_use ]
    pub fn group_get( &self, group : &str ) -> Option< Vec< Box< str > > >
    {
      self.scaled_nodes.get( group ).map( | ( n, _ ) | n ).cloned()
    }

    /// Get mutable reference to group nodes
    pub fn group_get_mut( &mut self, group : &str ) -> Option< &mut Vec< Box< str > > >
    {
      self.scaled_nodes.get_mut( group ).map( | ( n, _ ) | n )
    }

    /// Get reference to group scale
    #[ must_use ]
    pub fn scale_get( &self, group : &str ) -> Option< &F64x4 >
    {
      self.scaled_nodes.get( group ).map( | ( _, s ) | s )
    }

    /// Get mutable reference to group scale
    pub fn scale_get_mut( &mut self, group : &str ) -> Option< &mut F64x4 >
    {
      self.scaled_nodes.get_mut( group ).map( | ( _, s ) | s )
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
    fn scaled_rotation_apply
    (
      &self,
      node : &Rc< RefCell< Node > >,
      name : &str,
      scale : f64
    )
    {
      let Some( rotation ) = self.animation.get::< Sequence< Tween< QuatF64 > > >
      (
        &format!( "{name}{ROTATION_PREFIX}" )
      )
      else
      {
        return;
      };

      let mut tweens = rotation.players().to_vec();
      // Fix(BUG-198): `tweens` clones the persistent, already-playing Sequencer's own Tween
      // state -- each clone carries forward its REAL, cumulative `elapsed` ( already equal to
      // roughly `rotation.time()` for the currently active segment ), not a fresh `elapsed =
      // 0.0`. A few lines below, these clones are wrapped in a brand-new local `Sequence` and
      // immediately driven via `.update( rotation.time() )`, passing the FULL ABSOLUTE elapsed
      // time as though replaying from scratch -- correct for the local `Sequence`'s OWN
      // bookkeeping ( `Sequence::new` does start it fresh ), but doubling up on top of the
      // already-non-zero Tween-level elapsed underneath it. Visible result: every scaled
      // channel played at roughly double speed and froze at its segment's end pose once real
      // elapsed reached only half the segment's authored duration.
      // Root cause: `Sequence::new` intentionally never resets the players handed to it ( a
      // caller may legitimately want to seed it with already-in-progress players ) -- this
      // caller needed exactly that reset and never performed it.
      // Pitfall: only became externally observable once a precise ( not just "changed from
      // default" ) value assertion existed for a first-segment sample -- BUG-186's own
      // regression test only checks segment-boundary continuity ( equality between two
      // computed quaternions ), which stays correct regardless of the underlying elapsed being
      // wrong by a constant factor.
      for tween in &mut tweens
      {
        tween.reset();
      }
      let current = rotation.current_id_get();

      for i in 0..( ( current + 1 ).min( tweens.len() ) )
      {
        // Fix(BUG-186): continuity rebase must run for every segment after the first,
        // regardless of `scale` -- gating it on `scale < 1.0` left every segment after the
        // first sampling a stale, un-rebased `start_value` whenever `scale >= 1.0` (the GUI's
        // own default), producing a visible discontinuity at every segment boundary.
        if i > 0
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

      // Fix(BUG-185): this line unconditionally overwrote `tweens[ 0 ].start_value` with
      // `tweens.last().end_value` on every call, regardless of whether `current` ever reached
      // the last segment. `tweens` is rebuilt fresh from the unscaled Sequencer data every call
      // ( see `rotation.players().to_vec()` above ) and never persists across frames, so this
      // was never a meaningful "loop back to the start" write -- when `current` was the last
      // index the write was inert ( `tweens[ 0 ]` isn't sampled and is discarded next call
      // anyway ); when `current` was 0 ( the common case of playing a sequence's first segment )
      // it corrupted the CURRENTLY SAMPLED tween's `start_value` with the raw, un-rebased,
      // unscaled `end_value` of an unrelated, possibly-untouched last segment, producing a wrong
      // interpolated pose for the entire duration of the first segment.
      // Root cause: apparent leftover "seamless loop" logic that doesn't fit this function's
      // actual architecture -- `Sequence` has no automatic loop-back to segment 0, and even a
      // genuine external `.reset()` would be indistinguishable from first-ever playback here,
      // since `Scaler` holds no state to tell the two cases apart.
      // Pitfall: a write to `tweens[ 0 ]` placed after a loop that only touches `0..=current`
      // silently assumes `current` always reaches the last index by the time this line runs --
      // true only once per sequence lifetime at most, never on the far more common frames where
      // an earlier segment is still playing.

      let mut sequence= Sequence::new( tweens ).unwrap();
      sequence.update( rotation.time() );

      if let Some( tween ) = sequence.current_get()
      {
        let rotation = tween.value_get();
        let rotation = QuatF32::from( rotation.0.map( | v | v as f32 ) );
        node.borrow_mut().rotation_set( rotation );
      }
    }

    /// Applies scaled translation to a node based on the animation and scaling factor.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to apply the translation to
    /// * `name` - The name identifier for the node's translation animation
    /// * `scale` - The scaling factor to apply to each segment's translation delta
    // Fix(BUG-184): grouped nodes previously never had this channel applied at all --
    // `AnimatableComposition::set` only ever called `scaled_rotation_apply` for grouped nodes,
    // leaving `scaled_nodes`'s documented `x` ( transform ) weight component entirely unused.
    fn scaled_translation_apply
    (
      &self,
      node : &Rc< RefCell< Node > >,
      name : &str,
      scale : f64
    )
    {
      let Some( translation ) = self.animation.get::< Sequence< Tween< F64x3 > > >
      (
        &format!( "{name}{TRANSLATION_PREFIX}" )
      )
      else
      {
        return;
      };

      let mut tweens = translation.players().to_vec();
      // Fix(BUG-198): see `scaled_rotation_apply` -- same missing reset of the cloned,
      // already-elapsed Tween state before it's replayed via absolute time.
      for tween in &mut tweens
      {
        tween.reset();
      }
      let current = translation.current_id_get();

      for i in 0..( ( current + 1 ).min( tweens.len() ) )
      {
        if i > 0
        {
          tweens[ i ].start_value = tweens[ i - 1 ].end_value;
        }

        let prev = tweens[ i ].start_value;
        let curr = tweens[ i ].end_value;
        let delta = curr - prev;
        tweens[ i ].end_value = prev + delta * scale;
      }

      // Fix(BUG-185): see `scaled_rotation_apply` -- same unconditional, architecturally-dead/
      // harmful `tweens[ 0 ].start_value` overwrite, deleted for the same reason.

      let mut sequence = Sequence::new( tweens ).unwrap();
      sequence.update( translation.time() );

      if let Some( tween ) = sequence.current_get()
      {
        let translation = tween.value_get().0.map( | v | v as f32 );
        node.borrow_mut().translation_set( F32x3::from_array( translation ) );
      }
    }

    /// Applies scaled scale to a node based on the animation and scaling factor.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to apply the scale to
    /// * `name` - The name identifier for the node's scale animation
    /// * `scale` - The scaling factor to apply to each segment's scale delta
    // Fix(BUG-184): see `scaled_translation_apply` -- same previously-missing channel.
    fn scaled_scale_apply
    (
      &self,
      node : &Rc< RefCell< Node > >,
      name : &str,
      scale : f64
    )
    {
      let Some( scale_anim ) = self.animation.get::< Sequence< Tween< F64x3 > > >
      (
        &format!( "{name}{SCALE_PREFIX}" )
      )
      else
      {
        return;
      };

      let mut tweens = scale_anim.players().to_vec();
      // Fix(BUG-198): see `scaled_rotation_apply` -- same missing reset of the cloned,
      // already-elapsed Tween state before it's replayed via absolute time.
      for tween in &mut tweens
      {
        tween.reset();
      }
      let current = scale_anim.current_id_get();

      for i in 0..( ( current + 1 ).min( tweens.len() ) )
      {
        if i > 0
        {
          tweens[ i ].start_value = tweens[ i - 1 ].end_value;
        }

        let prev = tweens[ i ].start_value;
        let curr = tweens[ i ].end_value;
        let delta = curr - prev;
        tweens[ i ].end_value = prev + delta * scale;
      }

      // Fix(BUG-185): see `scaled_rotation_apply` -- same unconditional, architecturally-dead/
      // harmful `tweens[ 0 ].start_value` overwrite, deleted for the same reason.

      let mut sequence = Sequence::new( tweens ).unwrap();
      sequence.update( scale_anim.time() );

      if let Some( tween ) = sequence.current_get()
      {
        let scale_value = tween.value_get().0.map( | v | v as f32 );
        node.borrow_mut().scale_set( F32x3::from_array( scale_value ) );
      }
    }

    /// Applies unscaled transforms (translation, rotation, scale) to a node.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to apply transforms to
    /// * `name` - The name identifier for the node's animations
    fn unscaled_transforms_apply
    (
      &self,
      node : &Rc< RefCell< Node > >,
      name : &str
    )
    {
      if let Some( translation ) = self.animation.get::< Sequence< Tween< F64x3 > > >
      (
        &format!( "{name}{TRANSLATION_PREFIX}" )
      )
      {
        if let Some( translation ) = translation.current_get()
        {
          let translation = translation.value_get().0.map( | v | v as f32 );
          node.borrow_mut().translation_set( F32x3::from_array( translation ) );
        }
      }

      if let Some( rotation ) = self.animation.get::< Sequence< Tween< QuatF64 > > >
      (
        &format!( "{name}{ROTATION_PREFIX}" )
      )
      {
        if let Some( rotation ) = rotation.current_get()
        {
          let rotation = rotation.value_get().0.map( | v | v as f32 );
          node.borrow_mut().rotation_set( QuatF32::from( rotation ) );
        }
      }

      if let Some( scale ) = self.animation.get::< Sequence< Tween< F64x3 > > >
      (
        &format!( "{name}{SCALE_PREFIX}" )
      )
      {
        if let Some( scale ) = scale.current_get()
        {
          let scale = scale.value_get().0.map( | v | v as f32 );
          node.borrow_mut().scale_set( F32x3::from_array( scale ) );
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
          // Fix(BUG-184): translation/scale were never applied for grouped nodes at all --
          // only rotation had a `scaled_*_apply` implementation, so a grouped node's
          // `x`/`z` weight components were silently ignored and its translation/scale stayed
          // frozen at whatever the node's default state was.
          self.scaled_translation_apply( node, name, scales.x() );
          self.scaled_rotation_apply( node, name, scales.y() );
          self.scaled_scale_apply( node, name, scales.z() );
        }
      }

      // Apply unscaled transforms to remaining nodes
      for ( name, node ) in nodes
      {
        if !used_nodes.contains( name )
        {
          self.unscaled_transforms_apply( node, name );
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

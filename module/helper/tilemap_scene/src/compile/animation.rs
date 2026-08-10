// Time-calc casts (f32 → u32 / usize) are acceptable: time values are
// non-negative by construction (clamped/rem_euclid upstream) and small
// enough that integer range isn't a real risk.
#![ allow( clippy::cast_sign_loss, clippy::cast_possible_truncation ) ]

//! Time-based animation frame sampling.
//!
//! Given an [`Animation`] resource, the current global time, and the tile
//! position (for `HashCoord` phase offsets), [`resolve_animation_frame`]
//! returns the concrete `( asset, frame )` pair to draw this instant.
//!
//! See SPEC §7. All timing is deterministic: the same `( animation, time,
//! position )` triple always picks the same frame.

mod private
{
  use crate::compile::error::CompileError;
  use crate::hash::{ hash_coord, hash_str };
  use crate::resource::
  {
    Animation,
    AnimationMode,
    AnimationTiming,
    PhaseOffset,
    SpriteRef,
  };

  /// Pick the frame of `anim` that's active at `time_seconds` for a tile at
  /// `pos`, using the animation's declared `phase_offset`.
  ///
  /// `oneshot_origin` is the wall-clock time at which an `OneShot`
  /// animation began for *this instance* (typically
  /// `Instance.state_entered_time` so re-entering a `OneShot` state
  /// restarts the animation). It is ignored for `Loop` / `PingPong`
  /// animations — those are intentionally synchronised across
  /// instances via the master clock, with `phase_offset` providing
  /// per-instance jitter. Pass `0.0` from non-instance call sites
  /// (where `OneShot` start time is not meaningful) for the historical
  /// behaviour.
  ///
  /// Returns the `( asset_id, frame_name )` pair resolved to a single
  /// [`SpriteRef`] the caller can look up in the sprite id map.
  ///
  /// # Errors
  ///
  /// Returns [`CompileError::OutOfRange`] when the animation has no frames
  /// (degenerate declaration) or when a `FromSheet` variant addresses a
  /// non-existent index (caller is responsible for pre-allocating sprites
  /// in the asset-compile pass; here we just compute which sprite to pick).
  pub fn resolve_animation_frame
  (
    anim : &Animation,
    time_seconds : f32,
    oneshot_origin : f32,
    pos : ( i32, i32 ),
    instance_seed : Option< u32 >,
  ) -> Result< SpriteRef, CompileError >
  {
    let phase = phase_offset_seconds( anim, pos, instance_seed );
    // `OneShot` is per-instance triggered — its local time is the elapsed
    // wall-clock since this instance entered the state. Loop / PingPong
    // ride the master clock so neighbouring instances stay in harmonic
    // phase (jittered via `phase_offset`).
    let base = match anim.mode
    {
      AnimationMode::OneShot => time_seconds - oneshot_origin,
      _                      => time_seconds,
    };
    let local_t = base + phase;

    match &anim.timing
    {
      AnimationTiming::Regular { frames, fps } =>
      {
        if frames.is_empty()
        {
          return Err( CompileError::OutOfRange
          {
            owner : anim.id.clone(),
            index : 0,
            max : 0,
          });
        }
        let idx = pick_frame_index( local_t, *fps, frames.len(), anim.mode );
        Ok( frames[ idx ].clone() )
      },
      AnimationTiming::FromSheet { asset, start_frame, count, fps } =>
      {
        if *count == 0
        {
          return Err( CompileError::OutOfRange
          {
            owner : anim.id.clone(),
            index : 0,
            max : 0,
          });
        }
        let idx = pick_frame_index( local_t, *fps, *count as usize, anim.mode );
        let frame_name = ( *start_frame + idx as u32 ).to_string();
        Ok( SpriteRef { asset : asset.clone(), frame : frame_name } )
      },
      AnimationTiming::Irregular { frames } =>
      {
        if frames.is_empty()
        {
          return Err( CompileError::OutOfRange
          {
            owner : anim.id.clone(),
            index : 0,
            max : 0,
          });
        }
        // Total duration in seconds; clamp to OneShot's final frame for
        // times past the end.
        let total_duration_ms : u32 = frames.iter().map( | f | f.duration_ms ).sum();
        let total_duration_secs = total_duration_ms as f32 / 1000.0;

        let effective_t = match anim.mode
        {
          AnimationMode::Loop =>
          {
            if total_duration_secs > 0.0 { local_t.rem_euclid( total_duration_secs ) } else { 0.0 }
          },
          AnimationMode::PingPong =>
          {
            // Reflect around total_duration_secs → period 2*total_duration_secs.
            let period = 2.0 * total_duration_secs;
            if period > 0.0
            {
              let u = local_t.rem_euclid( period );
              if u > total_duration_secs { 2.0 * total_duration_secs - u } else { u }
            }
            else { 0.0 }
          },
          AnimationMode::OneShot => local_t.clamp( 0.0, total_duration_secs ),
        };

        let target_ms = ( effective_t * 1000.0 ) as u32;
        let mut accumulated = 0_u32;
        for frame in frames
        {
          accumulated = accumulated.saturating_add( frame.duration_ms );
          if accumulated > target_ms
          {
            return Ok( frame.sprite.clone() );
          }
        }
        // Fallthrough (time exactly at end, or OneShot clamped): last frame.
        Ok( frames[ frames.len() - 1 ].sprite.clone() )
      },
    }
  }

  /// Compute `phase_offset` in seconds for a given tile position. Thin
  /// wrapper retained for [`resolve_animation_frame`]; new callers use
  /// [`declared_phase_seconds`] directly.
  #[ inline ]
  fn phase_offset_seconds
  (
    anim : &Animation,
    pos : ( i32, i32 ),
    instance_seed : Option< u32 >,
  ) -> f32
  {
    declared_phase_seconds( anim, pos, instance_seed )
  }

  /// Resolve the animation's declared per-instance phase offset (in
  /// seconds) for an instance at grid coordinate `pos` (and, when
  /// known, with `instance_seed` from `Instance.instance_phase_seed`).
  ///
  /// Mirrors the renderer's frame-resolution path so completion-event
  /// detection in `Scene::tick` agrees byte-for-byte with what
  /// [`resolve_animation_frame`] would show on screen.
  #[ must_use ]
  pub fn declared_phase_seconds
  (
    anim : &Animation,
    pos : ( i32, i32 ),
    instance_seed : Option< u32 >,
  ) -> f32
  {
    match anim.phase_offset
    {
      PhaseOffset::None => 0.0,
      PhaseOffset::Fixed( s ) => s,
      PhaseOffset::HashCoord =>
      {
        let salt = hash_str( &anim.id );
        let raw = hash_coord( pos.0, pos.1, salt );
        let unit = ( raw as f32 ) / ( u32::MAX as f32 );
        // Multiply by the animation's *natural* period so neighbouring tiles
        // spread across the whole cycle, not just a tiny fraction of it.
        let period = animation_duration_seconds( anim );
        unit * period
      },
      PhaseOffset::Linear { per_q, per_r } =>
      {
        ( pos.0 as f32 ) * per_q + ( pos.1 as f32 ) * per_r
      },
      PhaseOffset::Instance =>
      {
        // Non-instance compile paths (edge / vertex / global passes)
        // don't have a per-instance seed; fall back to 0.0 so the
        // animation rides the master clock there.
        let Some( seed ) = instance_seed else { return 0.0 };
        // Mix the seed and the animation id through `hash_coord`'s
        // avalanche so neighbouring seeds (1, 2, 3 ...) land on
        // well-separated phases — XOR alone leaves the upper bits
        // unchanged and collapses unit-magnitude differences.
        let mixed = hash_coord( seed as i32, 0, hash_str( &anim.id ) );
        let unit = ( mixed as f32 ) / ( u32::MAX as f32 );
        let period = animation_duration_seconds( anim );
        unit * period
      },
    }
  }

  /// Total duration of one full play-through of the animation, in seconds.
  ///
  /// For `Regular { frames, fps }` = `frames.len() / fps`; for
  /// `FromSheet { count, fps }` = `count / fps`; for `Irregular`
  /// = the summed per-frame durations. Used by [`declared_phase_seconds`]
  /// to spread `HashCoord` phase across the natural cycle, and by
  /// `Scene::tick` to detect `OneShot` completion.
  #[ must_use ]
  pub fn animation_duration_seconds( anim : &Animation ) -> f32
  {
    match &anim.timing
    {
      AnimationTiming::Regular { frames, fps } =>
      {
        if *fps > 0.0 { frames.len() as f32 / fps } else { 0.0 }
      },
      AnimationTiming::FromSheet { count, fps, .. } =>
      {
        if *fps > 0.0 { *count as f32 / fps } else { 0.0 }
      },
      AnimationTiming::Irregular { frames } =>
      {
        let total_ms : u32 = frames.iter().map( | f | f.duration_ms ).sum();
        total_ms as f32 / 1000.0
      },
    }
  }

  /// Pick a regular-timing frame index from local time.
  fn pick_frame_index
  (
    local_t : f32,
    fps : f32,
    frame_count : usize,
    mode : AnimationMode,
  ) -> usize
  {
    if fps <= 0.0 || frame_count == 0
    {
      return 0;
    }
    let raw = ( local_t * fps ).max( 0.0 ) as usize;
    match mode
    {
      AnimationMode::Loop => raw % frame_count,
      AnimationMode::PingPong =>
      {
        if frame_count == 1 { return 0; }
        let period = 2 * ( frame_count - 1 );
        let cycle = raw % period;
        if cycle < frame_count { cycle } else { period - cycle }
      },
      AnimationMode::OneShot => raw.min( frame_count - 1 ),
    }
  }
}

mod_interface::mod_interface!
{
  exposed use resolve_animation_frame;
  own use animation_duration_seconds;
  own use declared_phase_seconds;
}

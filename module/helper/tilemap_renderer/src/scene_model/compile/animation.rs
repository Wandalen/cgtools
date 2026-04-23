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
  use crate::scene_model::compile::error::CompileError;
  use crate::scene_model::hash::{ hash_coord, hash_str };
  use crate::scene_model::resource::
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
    pos : ( i32, i32 ),
  ) -> Result< SpriteRef, CompileError >
  {
    let phase = phase_offset_seconds( anim, pos );
    let local_t = time_seconds + phase;

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
        Ok( SpriteRef( asset.clone(), frame_name ) )
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

  /// Compute `phase_offset` in seconds for a given tile position.
  fn phase_offset_seconds( anim : &Animation, pos : ( i32, i32 ) ) -> f32
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
        let period = animation_period_seconds( anim );
        unit * period
      },
    }
  }

  /// Natural period of the animation in seconds; used by `HashCoord` phase.
  fn animation_period_seconds( anim : &Animation ) -> f32
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

#[ cfg( test ) ]
mod tests
{
  use super::private::*;
  use crate::scene_model::resource::
  {
    Animation,
    AnimationMode,
    AnimationTiming,
    PhaseOffset,
    SpriteRef,
  };

  fn regular( id : &str, frames : &[ &str ], fps : f32, mode : AnimationMode ) -> Animation
  {
    Animation
    {
      id : id.into(),
      timing : AnimationTiming::Regular
      {
        frames : frames.iter().map( | f | SpriteRef( "a".into(), ( *f ).into() ) ).collect(),
        fps,
      },
      mode,
      phase_offset : PhaseOffset::None,
    }
  }

  #[ test ]
  fn regular_loop_wraps()
  {
    let a = regular( "w", &[ "0", "1", "2" ], 10.0, AnimationMode::Loop );
    let pick = | t | resolve_animation_frame( &a, t, ( 0, 0 ) ).unwrap().1;
    assert_eq!( pick( 0.0 ), "0" );
    assert_eq!( pick( 0.1 ), "1" );
    assert_eq!( pick( 0.25 ), "2" );
    assert_eq!( pick( 0.35 ), "0", "should have wrapped back to frame 0" );
  }

  #[ test ]
  fn one_shot_clamps()
  {
    let a = regular( "w", &[ "a", "b", "c" ], 10.0, AnimationMode::OneShot );
    let pick = | t | resolve_animation_frame( &a, t, ( 0, 0 ) ).unwrap().1;
    assert_eq!( pick( 0.0 ), "a" );
    assert_eq!( pick( 100.0 ), "c", "past end → stuck on last frame" );
  }

  #[ test ]
  fn pingpong_reflects()
  {
    let a = regular( "w", &[ "a", "b", "c" ], 10.0, AnimationMode::PingPong );
    let pick = | t | resolve_animation_frame( &a, t, ( 0, 0 ) ).unwrap().1;
    // Period = 2 * (3 - 1) = 4 ticks. Sequence: a b c b | a b c b | ...
    assert_eq!( pick( 0.00 ), "a" );
    assert_eq!( pick( 0.10 ), "b" );
    assert_eq!( pick( 0.20 ), "c" );
    assert_eq!( pick( 0.30 ), "b", "ping-ponged" );
    assert_eq!( pick( 0.40 ), "a" );
  }

  #[ test ]
  fn phase_offset_hashcoord_spreads_neighbours()
  {
    let mut a = regular( "w", &[ "0", "1", "2", "3" ], 4.0, AnimationMode::Loop );
    a.phase_offset = PhaseOffset::HashCoord;
    // Two neighbouring tiles, same global time — their local times should
    // differ (practically always) and so can their frames.
    let f_00 = resolve_animation_frame( &a, 0.0, ( 0, 0 ) ).unwrap().1;
    let f_10 = resolve_animation_frame( &a, 0.0, ( 1, 0 ) ).unwrap().1;
    // We can't assert inequality rigorously (hash could collide) but we can
    // sample many coords and check that at least SOME produce different frames.
    let samples : Vec< String > =
      ( 0..16 ).map( | q | resolve_animation_frame( &a, 0.0, ( q, 0 ) ).unwrap().1 ).collect();
    let unique_count = samples.iter().collect::< std::collections::HashSet< _ > >().len();
    assert!
    (
      unique_count >= 2,
      "phase-offset should spread neighbours across frames; samples: {samples:?} (first two {f_00} vs {f_10})",
    );
  }

  #[ test ]
  fn phase_offset_fixed_shifts_timeline()
  {
    let mut a = regular( "w", &[ "0", "1", "2" ], 10.0, AnimationMode::Loop );
    a.phase_offset = PhaseOffset::Fixed( 0.1 );
    // At global t=0, with phase=0.1s, we're 1 frame in.
    let frame = resolve_animation_frame( &a, 0.0, ( 0, 0 ) ).unwrap().1;
    assert_eq!( frame, "1" );
  }

  #[ test ]
  fn irregular_timing_honours_durations()
  {
    let a = Animation
    {
      id : "attack".into(),
      timing : AnimationTiming::Irregular
      {
        frames : vec!
        [
          crate::scene_model::resource::TimedFrame
          {
            sprite : SpriteRef( "a".into(), "wind_up".into() ),
            duration_ms : 100,
          },
          crate::scene_model::resource::TimedFrame
          {
            sprite : SpriteRef( "a".into(), "impact".into() ),
            duration_ms : 300,    // held
          },
          crate::scene_model::resource::TimedFrame
          {
            sprite : SpriteRef( "a".into(), "recover".into() ),
            duration_ms : 100,
          },
        ],
      },
      mode : AnimationMode::OneShot,
      phase_offset : PhaseOffset::None,
    };
    let pick = | t | resolve_animation_frame( &a, t, ( 0, 0 ) ).unwrap().1;
    assert_eq!( pick( 0.0  ), "wind_up" );
    assert_eq!( pick( 0.05 ), "wind_up" );
    assert_eq!( pick( 0.15 ), "impact" );
    assert_eq!( pick( 0.35 ), "impact", "still holding the accented frame" );
    assert_eq!( pick( 0.45 ), "recover" );
    assert_eq!( pick( 2.00 ), "recover", "OneShot clamps to last" );
  }
}

mod_interface::mod_interface!
{
  own use resolve_animation_frame;
}

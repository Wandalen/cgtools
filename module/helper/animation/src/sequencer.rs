//! Tools for managing [`AnimatablePlayer`] playback in every time moment
//!
//! This module provides two distinct playback coordinators, chosen based on how the underlying
//! players relate to each other:
//!
//! - [`Sequencer`]: a named, heterogeneous collection of independent [`AnimatablePlayer`]s (any
//!   mix of types) that all run in parallel, advanced by the same `update()` call every frame.
//!   Use it to coordinate multiple unrelated animations that play concurrently.
//! - [`Sequence`]: an ordered, homogeneous chain of same-type [`AnimatablePlayer`]s that play one
//!   at a time based on each player's `delay_get()`, like a timeline of consecutive clips. Use it
//!   to chain a strictly time-ordered series of animations of the same type.

mod private
{
  use rustc_hash::FxHashMap;
  use crate::
  {
    AnimatablePlayer, AnimationState
  };
  use error_tools::error;

  /// Sequencer for managing multiple animations with sequencing and grouping.
  // #[ derive( Debug ) ]
  pub struct Sequencer
  {
    /// Map of animation names to their animation behavior data
    players : FxHashMap< Box< str >, Box< dyn AnimatablePlayer > >,
    /// Current Sequencer time
    time : f64,
    /// Sequencer state
    state : AnimationState,
  }

  impl core::fmt::Debug for Sequencer
  {
    fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
    {
      f.debug_struct( "Sequencer" )
      .field("players", &self.players.len() )
      .field( "time", &self.time )
      .field( "state", &self.state )
      .finish()
    }
  }

  impl Clone for Sequencer
  {
    fn clone( &self ) -> Self
    {
      Self
      {
        players : self.players.iter()
        .map
        (
          | ( name, player ) |
          {
            ( name.clone(), clone_dyn_types::clone_into_box( player.as_ref() ) )
          }
        )
        .collect::< FxHashMap< _, _ > >(),
        time : self.time,
        state : self.state
      }
    }
  }

  impl Sequencer
  {
    /// Creates a new animation Sequencer.
    #[must_use]
    pub fn new() -> Self
    {
      Self
      {
        players : FxHashMap::default(),
        time : 0.0,
        state : AnimationState::Pending,
      }
    }

    /// Returns list of contained [`AnimatablePlayer`]'s names
    #[must_use]
    pub fn keys( &self ) -> Vec< Box< str > >
    {
      self.players.keys().cloned()
      .collect::< Vec< _ > >()
    }

    /// Inserts a [`AnimatablePlayer`] to the Sequencer.
    pub fn insert< T >( &mut self, name : &str, player : T )
    where T : AnimatablePlayer + 'static
    {
      self.players.insert( name.to_string().into(), Box::new( player ) );
      if self.state == AnimationState::Pending && !self.players.is_empty()
      {
        self.state = AnimationState::Running;
      }
    }

    /// Updates all animations in the Sequencer.
    pub fn update( &mut self, delta_time : f64 )
    {
      if self.state != AnimationState::Running
      {
        return;
      }

      self.time += delta_time;
      let mut all_completed = true;

      for player in self.players.values_mut()
      {
        player.update( delta_time );
        if !player.is_completed()
        {
          all_completed = false;
        }
      }

      if all_completed && !self.players.is_empty()
      {
        self.state = AnimationState::Completed;
      }
    }

    /// Gets reference to named player
    #[must_use]
    pub fn get< T >( &self, name : &str ) -> Option< &T >
    where T : AnimatablePlayer + 'static
    {
      let player_box = self.players.get( name )?;
      let any_ref = player_box.as_any();
      any_ref.downcast_ref::< T >()
    }

    /// Gets mutable reference to named player
    pub fn get_mut< T >( &mut self, name : &str ) -> Option< &mut T >
    where T : AnimatablePlayer + 'static
    {
      let player_box = self.players.get_mut( name )?;
      player_box.as_any_mut().downcast_mut::< T >()
    }

    /// Gets the current value of a named animation as dyn ref.
    #[must_use]
    pub fn get_dyn_value( &self, name : &str ) -> Option< &dyn AnimatablePlayer >
    {
      let player_box = self.players.get( name )?;
      Some( player_box.as_ref() )
    }

    /// Checks if the Sequencer has completed all animations.
    #[must_use]
    pub fn is_completed( &self ) -> bool
    {
      self.state == AnimationState::Completed
    }

    /// Pauses all animations in the Sequencer.
    pub fn pause( &mut self )
    {
      self.state = AnimationState::Paused;
      for player in self.players.values_mut()
      {
        player.pause();
      }
    }

    /// Resumes all animations in the Sequencer.
    pub fn resume( &mut self )
    {
      self.state = AnimationState::Running;
      for player in self.players.values_mut()
      {
        player.resume();
      }
    }

    /// Resets the  Sequencer and all animations.
    pub fn reset( &mut self )
    {
      self.time = 0.0;
      self.state = if self.players.is_empty()
      {
        AnimationState::Pending
      }
      else
      {
        AnimationState::Running
      };
      for player in self.players.values_mut()
      {
        player.reset();
      }
    }

    /// Renames an player in the Sequencer.
    pub fn rename_player( &mut self, current_name : &str, new_name : &str ) -> bool
    {
      if let Some( ( _, value ) ) = self.players.remove_entry( current_name )
      {
        self.players.insert( new_name.into(), value );
        true
      }
      else
      {
        false
      }
    }

    /// Removes an animation from the Sequencer.
    pub fn remove( &mut self, name : &str ) -> bool
    {
      self.players.remove( name ).is_some()
    }

    /// Gets the current  Sequencer time.
    #[must_use]
    pub fn time( &self ) -> f64
    {
      self.time
    }

    /// Gets the Sequencer state.
    #[must_use]
    pub fn state( &self ) -> AnimationState
    {
      self.state
    }

    /// Gets the number of active animations.
    #[must_use]
    pub fn animation_count( &self ) -> usize
    {
      self.players.len()
    }

    /// Progress of [`Sequencer`]
    #[must_use]
    pub fn progress( &self ) -> f64
    {
      if self.state == AnimationState::Pending
      {
        0.0
      }
      else
      {
        ( ( self.time() - self.delay_get() ) / self.duration_get() ).clamp( 0.0, 1.0 )
      }
    }

    /// Gets the longest duration among [`Self::players`], used as the Sequencer's overall
    /// animation duration in [`Self::progress`].
    #[must_use]
    pub fn duration_get( &self ) -> f64
    {
      let mut max_duration = 0.0;
      for player in self.players.values()
      {
        max_duration = player.duration_get().max( max_duration );
      }

      max_duration
    }

    // Fix(TASK-015): the reduction was seeded at f64::MAX (correct for a min-reduction) but then
    // called .max( min_delay ) instead of .min( min_delay ), so no real delay could ever displace
    // the seed — delay_get always returned f64::MAX, which made progress()'s
    // `( time - delay_get() ) / duration_get()` collapse to 0.0 after clamping, regardless of
    // actual elapsed time.
    // Root cause: wrong reduction direction — a max-reduction's comparison used against a
    // min-reduction's seed.
    // Pitfall: the return type and correct seed value are easy to eyeball as right; only the
    // comparison direction is wrong, so a glance at the seed alone gives false confidence.
    /// Get smallest delay of [`Self::players`]
    #[must_use]
    pub fn delay_get( &self ) -> f64
    {
      let mut min_delay = f64::MAX;
      for player in self.players.values()
      {
        min_delay = player.delay_get().min( min_delay );
      }

      min_delay
    }
  }

  impl Default for Sequencer
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  /// Error for handling wrong [`Sequence`] input data
  #[ non_exhaustive ]
  #[ derive( Debug, error::typed::Error ) ]
  pub enum SequenceError
  {
    /// Input players aren't sorted in time
    #[ error( "Input players aren't sorted by delay" ) ]
    Unsorted,
    /// Input players count isn't enough for animation
    #[ error( "Input players count isn't enough for animation" ) ]
    NotEnough
  }

  /// Sequence of [`AnimatablePlayer`]s of one type, played one at a time in delay order.
  ///
  /// Unlike [`Sequencer`], which runs a named, heterogeneous set of players in parallel,
  /// [`Sequence`] advances through a single ordered, homogeneous chain — only one player is
  /// active at any given elapsed time, selected by comparing elapsed time against each player's
  /// `delay_get()`.
  #[ derive( Debug, Clone ) ]
  pub struct Sequence< T >
  {
    /// Sequence of [`AnimatablePlayer`]s of one type
    players : Vec< T >,
    /// Current [`AnimatablePlayer`] index
    current : usize,
    /// Animation duration in seconds
    duration : f64,
    /// Current elapsed time
    elapsed : f64,
    /// Current animation state
    state : AnimationState,
    /// Delay before animation starts
    delay : f64,
  }

  impl< T > Sequence< T >
  where T : AnimatablePlayer + 'static
  {
    /// [`Sequence`] constructor
    ///
    /// # Errors
    /// Returns [`SequenceError::NotEnough`] if fewer than two players are provided, or
    /// [`SequenceError::Unsorted`] if the players aren't ordered by non-decreasing `delay_get()`.
    ///
    /// # Panics
    /// Never panics in practice: the length check above guarantees at least two players, so the
    /// `players.first()`/`players.last()` unwraps below always succeed.
    pub fn new( mut players : Vec< T > ) -> Result< Self, SequenceError >
    {
      if players.len() < 2
      {
        return Err( SequenceError::NotEnough );
      }

      // Fix(TASK-015): `last_delay` was declared immutable and never reassigned inside the loop,
      // so every iteration compared against the initial 0.0 instead of the previous player's
      // delay, making the Unsorted check fire only for a negative delay (which delay_get() never
      // produces) — the validation was effectively dead code regardless of actual player order.
      // Root cause: missing `last_delay = player.delay_get();` update at the end of the loop body.
      // Pitfall: the check reads as correct at a glance (right comparison, right error) — only
      // the absence of the reassignment reveals it can never trigger on realistic input.
      let mut last_delay = 0.0;
      for player in &mut players
      {
        if last_delay > player.delay_get()
        {
          return Err( SequenceError::Unsorted );
        }
        last_delay = player.delay_get();
      }

      let delay = players.first().unwrap().delay_get();
      let player = players.last().unwrap();
      let duration = player.delay_get() + player.duration_get() - delay;

      Ok
      (
        Self
        {
          players,
          current : 0,
          duration,
          elapsed : 0.0,
          state : AnimationState::Pending,
          delay
        }
      )
    }

    /// Returns active [`AnimatablePlayer`] at current elapsed time
    #[must_use]
    pub fn current_get( &self ) -> Option< &T >
    {
      self.players.get( self.current )
    }

    /// Returns active [`AnimatablePlayer`] index in players array
    #[must_use]
    pub fn current_id_get( &self ) -> usize
    {
      self.current
    }

    /// Returns reference to all sequence of players
    #[must_use]
    pub fn players( &self ) -> &[ T ]
    {
      &self.players
    }

    /// Returns mutable reference to all sequence of players
    pub fn players_mut( &mut self ) -> &mut Vec< T >
    {
      &mut self.players
    }

    /// Returns elapsed time
    #[must_use]
    pub fn time( &self ) -> f64
    {
      self.elapsed
    }
  }

  impl< T > AnimatablePlayer for Sequence< T >
  where T : AnimatablePlayer + Clone + 'static
  {
    fn update( &mut self, delta_time : f64 )
    {
      if self.state == AnimationState::Completed || self.state == AnimationState::Paused
      {
        return;
      }

      self.elapsed += delta_time;

      let index = self.players.binary_search_by
      (
        | player |
        {
          player.delay_get().partial_cmp( &self.elapsed ).expect( "Animation keyframes can't be NaN" )
        }
      );

      let index = match index
      {
        Ok( id ) | Err( id ) => id
      };

      let mut current_id = index;

      if index >= self.players.len()
      {
        current_id = self.players.len().saturating_sub( 1 );
      }

      match self.current.cmp( &current_id )
      {
        core::cmp::Ordering::Equal =>
        {
          let Some( current ) = self.players.get_mut( self.current )
          else
          {
            return;
          };
          let old_elapsed = current.delay_get() + ( current.progress() * current.duration_get() );
          current.update( old_elapsed + delta_time );
        },
        core::cmp::Ordering::Less =>
        {
          self.current = current_id;
          let Some( current ) = self.players.get_mut( self.current )
          else
          {
            return;
          };
          current.update( self.elapsed );
        },
        core::cmp::Ordering::Greater =>
        {
          // Only arises from a negative `delta_time` (elapsed time moving backward past the
          // active player) — not a case this frame-forward sequencer supports. No switch or
          // update happens; a subsequent forward-moving frame naturally re-synchronizes
          // `current_id` upward and normal playback resumes.
        }
      }

      let Some( current ) = self.players.get_mut( self.current )
      else
      {
        return;
      };

      match self.state
      {
        AnimationState::Pending if self.elapsed - current.delay_get() > 0.0 =>
        {
          self.state = AnimationState::Running;
        },
        AnimationState::Running
        if self.current >= self.players.len() - 1 &&
        self.players.get( self.current ).map_or( true, AnimatablePlayer::is_completed ) =>
        {
          self.state = AnimationState::Completed;
        },
        _ => {}
      }
    }

    fn is_completed( &self ) -> bool
    {
      self.state == AnimationState::Completed
    }

    fn pause( &mut self )
    {
      if self.state == AnimationState::Running
      {
        self.state = AnimationState::Paused;
      }

      for player in &mut self.players { player.pause() }
    }

    fn resume( &mut self )
    {
      if self.state == AnimationState::Paused
      {
        self.state = AnimationState::Running;
      }

      for player in &mut self.players { player.resume() }
    }

    fn reset( &mut self )
    {
      self.current = 0;
      self.state = AnimationState::Pending;
      self.elapsed = 0.0;

      for player in &mut self.players { player.reset() }
    }

    fn duration_get( &self ) -> f64
    {
      self.duration
    }

    fn delay_get( &self ) -> f64
    {
      self.delay
    }

    fn progress( &self ) -> f64
    {
      if self.state == AnimationState::Pending
      {
        0.0
      }
      else
      {
        ( ( self.elapsed - self.delay ) / self.duration ).clamp( 0.0, 1.0 )
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
  }
}

crate::mod_interface!
{
  orphan use
  {
    Sequencer,
    Sequence,
    SequenceError
  };
}

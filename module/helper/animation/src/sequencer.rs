//!

mod private
{
  /// Sequencer for managing multiple animations with sequencing and grouping.
  #[derive(Debug)]
  pub struct Sequencer {
    /// Map of animation names to their tween data
    tweens: HashMap<String, Box<dyn AnimatableValue>>,
    /// Current Sequencer time
    time: f32,
    /// Sequencer state
    state: AnimationState,
  }

  impl Sequencer {
    /// Creates a new animation Sequencer.
    pub fn new() -> Self {
      Self {
        tweens: HashMap::new(),
        time: 0.0,
        state: AnimationState::Pending,
      }
    }

    /// Adds a tween to the Sequencer.
    pub fn add_tween<T: Animatable + 'static>(&mut self, name: &str, tween: Tween<T>) {
      self.tweens.insert(name.to_string(), Box::new(tween));
      if self.state == AnimationState::Pending && !self.tweens.is_empty() {
        self.state = AnimationState::Running;
      }
    }

    /// Updates all animations in the Sequencer.
    pub fn update(&mut self, delta_time: f32) {
      if self.state != AnimationState::Running {
        return;
      }

      self.time += delta_time;
      let mut all_completed = true;

      for tween in self.tweens.values_mut() {
        tween.update(delta_time);
        if !tween.is_completed() {
          all_completed = false;
        }
      }

      if all_completed && !self.tweens.is_empty() {
        self.state = AnimationState::Completed;
      }
    }

    /// Gets the current value of a named animation.
    pub fn get_value<T: Animatable + 'static>(&self, name: &str) -> Option<T> {
      let tween_box = self.tweens.get(name)?;
      let any_ref = tween_box.as_any();
      if let Some(concrete_tween) = any_ref.downcast_ref::<Tween<T>>() {
        Some(concrete_tween.get_current_value())
      } else {
        None
      }
    }

    /// Checks if the Sequencer has completed all animations.
    pub fn is_completed(&self) -> bool {
      self.state == AnimationState::Completed
    }

    /// Pauses all animations in the Sequencer.
    pub fn pause(&mut self) {
      self.state = AnimationState::Paused;
      for tween in self.tweens.values_mut() {
        tween.pause();
      }
    }

    /// Resumes all animations in the Sequencer.
    pub fn resume(&mut self) {
      self.state = AnimationState::Running;
      for tween in self.tweens.values_mut() {
        tween.resume();
      }
    }

    /// Resets the  Sequencer and all animations.
    pub fn reset(&mut self) {
      self.time = 0.0;
      self.state = if self.tweens.is_empty() {
        AnimationState::Pending
      } else {
        AnimationState::Running
      };
      for tween in self.tweens.values_mut() {
        tween.reset();
      }
    }

    /// Removes an animation from the Sequencer.
    pub fn remove_tween(&mut self, name: &str) -> bool {
      self.tweens.remove(name).is_some()
    }

    /// Gets the current  Sequencer time.
    pub fn time(&self) -> f32 {
      self.time
    }

    /// Gets the  Sequencer state.
    pub fn state(&self) -> AnimationState {
      self.state
    }

    /// Gets the number of active animations.
    pub fn animation_count(&self) -> usize {
      self.tweens.len()
    }
  }

  impl Default for  Sequencer {
    fn default() -> Self {
      Self::new()
    }
  }

  /// Trait for type-erased animatable values in Sequencer.
  pub trait AnimatableValue: std::fmt::Debug {
    fn update(&mut self, delta_time: f32);
    fn is_completed(&self) -> bool;
    fn pause(&mut self);
    fn resume(&mut self);
    fn reset(&mut self);
    fn as_any(&self) -> &dyn std::any::Any;
  }
}

crate::mod_interface!
{
  orphan use
  {
    AnimatableValue,
    Sequencer
  };
}
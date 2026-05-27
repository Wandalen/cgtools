//! Events emitted by [`crate::scene::Scene::tick`].
//!
//! The retained-mode `Scene` is the source of truth for animation
//! lifetimes, so completion-style notifications belong here rather
//! than at the renderer level — multiple renderers can drive the same
//! scene, but the "this `OneShot` just ended" fact is intrinsically per
//! scene, not per renderer.
//!
//! The enum is `#[ non_exhaustive ]` from day one; later steps may add
//! batch-rebuild, spawn / despawn, or state-transition events without
//! breaking match expressions.

mod private
{
  use crate::instance::{ InstanceHandle, StateHandle };
  use crate::resource::AnimationRef;

  /// One observation produced by [`crate::scene::Scene::tick`].
  #[ derive( Debug, Clone ) ]
  #[ non_exhaustive ]
  pub enum SceneEvent
  {
    /// A `OneShot` animation just crossed its total duration on an
    /// instance during the most recent `tick` call.
    ///
    /// Fires exactly once per crossing.
    /// [`crate::scene::Scene::set_state`] resets `state_entered_time`
    /// to the current clock, restarting the `OneShot` from frame 0 and
    /// re-arming the event — calling `set_state` is the correct way to
    /// replay attack / death / pulse animations on long-lived instances.
    AnimationCompleted
    {
      /// Instance whose animation completed.
      instance : InstanceHandle,
      /// State that was active when the animation completed. Identifies
      /// *which* layer stack the completed layer belongs to — useful
      /// when an instance has different `OneShot`s in different states.
      state : StateHandle,
      /// 0-based index into the state's layer stack — disambiguates
      /// multiple `OneShot` layers that share the same state.
      layer_index : u16,
      /// The animation that finished, by [`AnimationRef`] id.
      animation : AnimationRef,
    },
  }
}

mod_interface::mod_interface!
{
  exposed use SceneEvent;
}

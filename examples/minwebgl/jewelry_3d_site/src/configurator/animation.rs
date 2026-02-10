use std::{ cell::RefCell, rc::Rc };
use renderer::webgl::Material;
use rustc_hash::FxHashMap;
use ::animation::{ AnimatablePlayer, Sequencer };

/// Type alias for material animation callbacks
pub type MaterialAnimationCallback = fn ( &dyn AnimatablePlayer, &Rc< RefCell< Box< dyn Material > > > );

/// Controls and updates animations and then applies interpolated values to materials using callbacks
#[ non_exhaustive ]
pub struct AnimationState
{
  /// Animation storage, player and state manager
  pub animations : Sequencer,
  /// Material updated in callbacks
  pub materials: FxHashMap< String, Rc< RefCell< Box< dyn Material > > > >,
  /// Callbacks triggered when animations are updated in [`AnimationState::animations`].
  /// Callbacks can update values inside material
  pub material_callbacks : FxHashMap< String, MaterialAnimationCallback >
}

impl Default for AnimationState
{
  #[ inline ]
  fn default() -> Self
  {
    Self::new()
  }
}

impl AnimationState
{
  /// Creates a new instance
  #[ must_use ]
  #[ inline ]
  pub fn new() -> Self
  {
    let mut animations = Sequencer::new();
    animations.resume();

    Self
    {
      animations,
      materials : FxHashMap::default(),
      material_callbacks : FxHashMap::default()
    }
  }

  /// Updates animations, calls callbacks and removes completed animations
  #[ inline ]
  pub fn update( &mut self, delta_time : f64 )
  {
    self.animations.resume();
    self.animations.update( delta_time );

    for name in self.animations.keys()
    {
      let name = name.as_ref();
      let Some( callback ) = self.material_callbacks.get( name )
      else
      {
        continue;
      };

      let Some( material ) = self.materials.get( name )
      else
      {
        continue;
      };

      if let Some( player ) = self.animations.get_dyn_value( name.as_ref() )
      {
        callback( player, material );
      }
    }

    for name in self.animations.keys()
    {
      let completed = if let Some( player ) = self.animations.get_dyn_value( name.as_ref() )
      {
        player.is_completed()
      }
      else
      {
        continue;
      };

      if completed
      {
        self.animations.remove( name.as_ref() );
        self.materials.remove( name.as_ref() );
        self.material_callbacks.remove( name.as_ref() );
      }
    }
  }

  /// Adds new animations with callbacks
  #[ inline ]
  pub fn add_material_animation< P >
  (
    &mut self,
    material : &Rc< RefCell< Box< dyn Material > > >,
    player : P,
    callback : MaterialAnimationCallback
  )
  where P : AnimatablePlayer + 'static
  {
    let name = material.borrow().get_id().to_string();

    self.animations.insert::< P >( &name, player );
    if self.animations.is_completed()
    {
      self.animations.resume();
    }
    self.materials.insert( name.clone(), material.clone() );
    self.material_callbacks.insert( name, callback );
  }
}

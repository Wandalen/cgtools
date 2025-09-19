mod private
{
  use std::
  {
    cell::RefCell,
    collections::HashMap,
    rc::Rc
  };
  use animation::
  {
    Sequence,
    Sequencer,
    Tween
  };
  use minwebgl as gl;
  use gl::{ F64x3, F32x3, QuatF32, QuatF64 };
  use crate::webgl::Node;

  /// Prefix used for getting [`Node`] translation
  pub const TRANSLATION_PREFIX : &'static str = ".translation";
  /// Prefix used for getting [`Node`] rotation
  pub const ROTATION_PREFIX : &'static str = ".rotation";
  /// Prefix used for getting [`Node`] scale
  pub const SCALE_PREFIX : &'static str = ".scale";

  /// Gives opportunity to change [`Node`]'s transforms in any way
  /// Interface used in [`Animation`] for using complex animation behaviours.
  pub trait AnimatableComposition
  {
    /// Updates all underlying [`animation::AnimatablePlayer`]'s
    fn update( &mut self, delta_time : f64 );

    /// Sets all simple 3D transformations for every
    /// [`Node`] related to this [`AnimatableComposition`]
    fn set( &self, nodes : &HashMap< Box< str >, Rc< RefCell< Node > > > );

    /// Returns a type-erased reference to the underlying value.
    fn as_any( &self ) -> &dyn core::any::Any;

    /// Returns a type-erased mutable reference to the underlying value.
    fn as_any_mut( &mut self ) -> &mut dyn core::any::Any;
  }

  impl AnimatableComposition for Sequencer
  {
    fn update( &mut self, delta_time : f64 )
    {
      Sequencer::update( self, delta_time );
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
        if let Some( translation ) = self.get::< Sequence< Tween< F64x3 > > >
        (
          &format!( "{}{}", name, TRANSLATION_PREFIX )
        )
        {
          if let Some( translation ) = translation.get_current()
          {
            let translation = translation.get_value().0.map( | v | v as f32 );
            node.borrow_mut().set_translation( F32x3::from_array( translation ) );
          }
        }

        if let Some( rotation ) = self.get::< Sequence< Tween< QuatF64 > > >
        (
          &format!( "{}{}", name, ROTATION_PREFIX )
        )
        {
          if let Some( rotation ) = rotation.get_current()
          {
            let rotation = rotation.get_value().0.map( | v | v as f32 );
            node.borrow_mut().set_rotation( QuatF32::from( rotation ) );
          }
        }

        if let Some( scale ) = self.get::< Sequence< Tween< F64x3 > > >
        (
          &format!( "{}{}", name, SCALE_PREFIX )
        )
        {
          if let Some( scale ) = scale.get_current()
          {
            let scale = scale.get_value().0.map( | v | v as f32 );
            node.borrow_mut().set_scale( F32x3::from_array( scale ) );
          }
        }
      }
    }
  }

  /// 3D transformation data including translation, rotation, and scale components.
  pub struct Transform
  {
    /// Translation
    pub translation : F64x3,
    /// Rotation
    pub rotation : QuatF64,
    /// Scale
    pub scale : F64x3,
  }

  /// Contains data for animating [`Mesh`]
  #[ derive( Clone ) ]
  pub struct Animation
  {
    /// Animation name
    pub name : Option< Box< str > >,
    /// Animation behavior
    pub animation : Rc< RefCell< Box< dyn AnimatableComposition > > >,
    /// Related animated [`Node`]'s
    pub nodes : HashMap< Box< str >, Rc< RefCell< Node > > >
  }

  impl Animation
  {
    /// Updates underlying [`AnimatableComposition`] for current [`Animation`]
    pub fn update( &mut self, delta_time : f64 )
    {
      self.animation.borrow_mut().update( delta_time.into() );
    }

    /// Sets all simple 3D transformations for every
    /// [`Node`] related to this [`Animation`]
    pub fn set( &self )
    {
      self.animation.borrow().set( &self.nodes );
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    AnimatableComposition,
    Animation,
    Transform
  };

  own use
  {
    TRANSLATION_PREFIX,
    ROTATION_PREFIX,
    SCALE_PREFIX
  };
}

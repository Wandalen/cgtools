mod private
{
  use rustc_hash::FxHashMap;
  use std::{ rc::Rc, cell::RefCell };
  use animation::
  {
    Animatable, AnimatablePlayer, Sequence, Sequencer, Tween
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

  /// Makes smooth transition between start and and [`Sequencer`]
  /// that can be configured by tween
  #[ derive( Clone ) ]
  pub struct Transition
  {
    /// Animation from which transition takes place
    start : Sequencer,
    /// Animation to which transition takes place
    end : Sequencer,
    /// Transition behavior. Using this [`Tween`] delay, duration,
    /// start, end and easing of transition parameter can be configured
    tween : Tween< f64 >
  }

  impl Transition
  {
    /// Create new [`Transition`]
    pub fn new
    (
      start : Sequencer,
      end : Sequencer,
      tween : Tween< f64 >
    )
    -> Self
    {
      Self
      {
        start,
        end,
        tween
      }
    }

    /// Get reference to underlying start [`Sequencer`]
    pub fn start_as_ref( &self ) -> &Sequencer
    {
      &self.start
    }

    /// Get mutable reference to underlying start [`Sequencer`]
    pub fn start_as_mut( &mut self ) -> &mut Sequencer
    {
      &mut self.start
    }

    /// Get reference to underlying end [`Sequencer`]
    pub fn end_as_ref( &self ) -> &Sequencer
    {
      &self.end
    }

    /// Get mutable reference to underlying end [`Sequencer`]
    pub fn end_as_mut( &mut self ) -> &mut Sequencer
    {
      &mut self.end
    }

    /// Get reference to underlying [`Tween< f64 >`]
    pub fn tween_get( &self ) -> &Tween< f64 >
    {
      &self.tween
    }

    /// Reset [`Transition`]
    pub fn reset( &mut self )
    {
      self.start.reset();
      self.end.reset();
      self.tween.reset();
    }
  }

  impl AnimatableComposition for Transition
  {
    /// Updates all underlying [`animation::AnimatablePlayer`]'s
    fn update( &mut self, delta_time : f64 )
    {
      if self.start.is_completed()
      {
        self.start.reset();
      }

      if self.end.is_completed()
      {
        self.end.reset();
      }

      self.start.update( delta_time );
      self.end.update( delta_time );
      self.tween.update( delta_time );
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
      let t = self.tween.value_get();

      for ( name, node ) in nodes
      {
        let ( mut a, mut b ) = ( None, None );
        if let Some( translation ) = self.start.get::< Sequence< Tween< F64x3 > > >
        (
          &format!( "{}{}", name, TRANSLATION_PREFIX )
        )
        {
          if let Some( translation ) = translation.current_get()
          {
            let translation = translation.value_get().0.map( | v | v as f32 );
            a = Some( F32x3::from_array( translation ) );
          }
        }

        if let Some( translation ) = self.end.get::< Sequence< Tween< F64x3 > > >
        (
          &format!( "{}{}", name, TRANSLATION_PREFIX )
        )
        {
          if let Some( translation ) = translation.current_get()
          {
            let translation = translation.value_get().0.map( | v | v as f32 );
            b = Some( F32x3::from_array( translation ) );
          }
        }

        let translation = match ( a, b )
        {
          ( Some( a ), Some( b ) ) =>
          {
            Some( a.interpolate( &b, t ) )
          },
          ( Some( a ), None ) => Some( a ),
          ( None, Some( b ) ) => Some( b ),
          ( None, None ) => None
        };
        if let Some( translation ) = translation
        {
          node.borrow_mut().set_translation( translation );
        }

        let ( mut a, mut b ) = ( None, None );
        if let Some( rotation ) = self.start.get::< Sequence< Tween< QuatF64 > > >
        (
          &format!( "{}{}", name, ROTATION_PREFIX )
        )
        {
          if let Some( rotation ) = rotation.current_get()
          {
            let rotation = rotation.value_get().0.map( | v | v as f32 );
            a = Some( QuatF32::from( rotation ) );
          }
        }

        if let Some( rotation ) = self.end.get::< Sequence< Tween< QuatF64 > > >
        (
          &format!( "{}{}", name, ROTATION_PREFIX )
        )
        {
          if let Some( rotation ) = rotation.current_get()
          {
            let rotation = rotation.value_get().0.map( | v | v as f32 );
            b = Some( QuatF32::from( rotation ) );
          }
        }

        let rotation = match ( a, b )
        {
          ( Some( a ), Some( b ) ) =>
          {
            Some( a.interpolate( &b, t ) )
          },
          ( Some( a ), None ) => Some( a ),
          ( None, Some( b ) ) => Some( b ),
          ( None, None ) => None
        };

        if let Some( rotation ) = rotation
        {
          node.borrow_mut().set_rotation( rotation );
        }

        let ( mut a, mut b ) = ( None, None );
        if let Some( scale ) = self.start.get::< Sequence< Tween< F64x3 > > >
        (
          &format!( "{}{}", name, SCALE_PREFIX )
        )
        {
          if let Some( scale ) = scale.current_get()
          {
            let scale = scale.value_get().0.map( | v | v as f32 );
            a = Some( F32x3::from_array( scale ) );
          }
        }

        if let Some( scale ) = self.end.get::< Sequence< Tween< F64x3 > > >
        (
          &format!( "{}{}", name, SCALE_PREFIX )
        )
        {
          if let Some( scale ) = scale.current_get()
          {
            let scale = scale.value_get().0.map( | v | v as f32 );
            b = Some( F32x3::from_array( scale ) );
          }
        }

        let scale = match ( a, b )
        {
          ( Some( a ), Some( b ) ) =>
          {
            Some( a.interpolate( &b, t ) )
          },
          ( Some( a ), None ) => Some( a ),
          ( None, Some( b ) ) => Some( b ),
          ( None, None ) => None
        };

        if let Some( scale ) = scale
        {
          node.borrow_mut().set_scale( scale );
        }
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Transition
  };
}

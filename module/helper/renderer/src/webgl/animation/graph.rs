mod private
{
  use crate::
  {
    Transition,
    AnimatableComposition
  };

  ///
  struct AnimationGraph
  {
    current :
    ///
    animations : Vec< Transition >,
  }

  impl AnimationGraph
  {
    fn new() -> Self
    {
      Self
      {

      }
    }

    fn add( &mut self, name : Box< str >, transition : Transition )
    {
      self.animations.insert( name, transition );
    }

    fn remove( &mut self, name : Box< str > )
    {
      self.animations.remove( name );
    }
  }

  impl AnimatableComposition for AnimationGraph
  {
    fn update( &mut self, delta_time : f64 )
    {

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
        if let Some( translation ) = self.end.get::< Sequence< Tween< F64x3 > > >
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

        if let Some( rotation ) = self.end.get::< Sequence< Tween< QuatF64 > > >
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

        if let Some( scale ) = self.end.get::< Sequence< Tween< F64x3 > > >
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
}

crate::mod_interface!
{
  orphan use
  {
    AnimationGraph
  };
}

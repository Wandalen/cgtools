mod private
{
  use std::collections::{ HashMap, HashSet };
  use std::{ rc::Rc, cell::RefCell };
  use animation::
  {
    Tween,
    Sequence,
    Sequencer
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

  /// Animation modifier that can scale animation for different groups of related [`Node`]'s
  #[ derive( Clone ) ]
  pub struct Scaler
  {
    /// Animation that must be scaled
    pub animation : Sequencer,
    /// Set of grouped [`Node`]'s with their scaling weights for
    /// each simple 3D transofrmation. Weights vector consist of
    /// such components:
    /// - x - transform
    /// - y - rotation
    /// - z - scale
    scaled_nodes : HashMap< Box< str >, ( Vec< Box< str > >, F64x3 ) >
  }

  impl Scaler
  {
    /// Create new [`Scaler`]
    pub fn new( animation : Sequencer ) -> Self
    {
      Self
      {
        animation,
        scaled_nodes : HashMap::new()
      }
    }

    /// Add scaled nodes group
    pub fn add
    (
      &mut self,
      group_name : Box< str >,
      node_names : Vec< Box< str > >,
      scale : F64x3
    )
    {
      self.scaled_nodes.insert( group_name, ( node_names, scale ) );
    }

    /// Remove scaled nodes group
    pub fn remove( &mut self, group_name : Box< str > )
    {
      self.scaled_nodes.remove( &group_name );
    }

    /// Get reference to underlying [`Sequencer`] by name
    pub fn get_animation( &self ) -> &Sequencer
    {
      &self.animation
    }

    /// Get mutable reference to underlying [`Sequencer`] by name
    pub fn get_animation_mut( &mut self ) -> &mut Sequencer
    {
      &mut self.animation
    }
  }

  impl AnimatableComposition for Scaler
  {
    fn update( &mut self, delta_time : f64 )
    {
      self.animation.update( delta_time );
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
      let mut used_nodes = HashSet::< Box< str > >::new();

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

          if let Some( translation ) = self.animation.get::< Sequence< Tween< F64x3 > > >
          (
            &format!( "{}{}", name, TRANSLATION_PREFIX )
          )
          {
            if let Some( tween ) = translation.get_current()
            {
              let s = scales.x();
              // let delta = tween.get_value() - tween.start_value;
              // let scaled_delta = delta * s;
              // let translation = tween.start_value + scaled_delta;
              let translation = tween.get_value() * s;
              let translation = F32x3::from_array( translation.0.map( | v | v as f32 ) );
              node.borrow_mut().set_translation( translation );
            }
          }

          if let Some( rotation ) = self.animation.get::< Sequence< Tween< QuatF64 > > >
          (
            &format!( "{}{}", name, ROTATION_PREFIX )
          )
          {
            if let Some( tween ) = rotation.get_current()
            {
              let s = scales.y();
              // let delta = tween.get_value() - tween.start_value;
              // let scaled_delta = delta * s;
              // let rotation = tween.start_value + scaled_delta;
              let rotation = ( tween.get_value() * s ).normalize();
              let rotation = QuatF32::from( rotation.0.map( | v | v as f32 ) );
              node.borrow_mut().set_rotation( rotation );
            }
          }

          if let Some( scale ) = self.animation.get::< Sequence< Tween< F64x3 > > >
          (
            &format!( "{}{}", name, SCALE_PREFIX )
          )
          {
            if let Some( tween ) = scale.get_current()
            {
              let s = scales.z();
              // let delta = tween.get_value() - tween.start_value;
              // let scaled_delta = delta * s;
              // let scale = tween.start_value + scaled_delta;
              let scale = tween.get_value() * s;
              let scale = F32x3::from_array( scale.0.map( | v | v as f32 ) );
              node.borrow_mut().set_translation( scale );
            }
          }
        }
      }

      for ( name, node ) in nodes
      {
        if !used_nodes.contains( name )
        {
          if let Some( translation ) = self.animation.get::< Sequence< Tween< F64x3 > > >
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

          if let Some( rotation ) = self.animation.get::< Sequence< Tween< QuatF64 > > >
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

          if let Some( scale ) = self.animation.get::< Sequence< Tween< F64x3 > > >
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
}

crate::mod_interface!
{
  orphan use
  {
    Scaler
  };
}

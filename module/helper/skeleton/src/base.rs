//!

mod private
{
  use bytemuck::cast_slice;
  use minwebgl as gl;

  use gl::{ GL, F32x4x4 };
  use std::{ cell::RefCell, collections::HashMap, rc::Rc };
  use animation::
  {
    interpolation::Transform,
    sequencer::Sequencer,
  };

  /// 
  pub struct Skeleton
  {
    nodes : Vec< Box< str > >,
    inverse_bind_matrices :  Vec< F32x4x4 >,
    animation : Option< Rc< RefCell< Sequencer > > >
  }

  impl Skeleton
  {
    /// 
    fn new( nodes : Vec< Box< str > >, inverse_bind_matrices : Vec< F32x4x4 > ) -> Self
    {
      Self
      {
        nodes,
        inverse_bind_matrices,
        animation : None
      }
    }

    /// 
    fn set_animation( &mut self, animation : Option< &Rc< RefCell< Sequencer > > > )
    {
      self.animation = animation.cloned();
    }

    /// 
    fn get_animation( &self ) -> Option< Rc< RefCell< Sequencer > > >
    {
      self.animation.clone()
    }

    /// 
    fn update( &self, t : f32 ) -> HashMap< Box< str >, Transform >
    {
      
    }

    /// 
    fn upload( &self, gl : &GL ) 
    {

    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Skeleton
  };
}
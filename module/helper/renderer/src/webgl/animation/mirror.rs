mod private
{
  use rustc_hash::{ FxHashMap, FxHashSet };
  use std::{ rc::Rc, cell::RefCell };
  use animation::
  {
    Tween,
    Sequence,
    Sequencer,
    AnimatablePlayer
  };
  use mingl as gl;
  use gl::{ F32x3, F64x3, F64x4, QuatF32, QuatF64 };
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

  #[ derive( Clone, Copy, Debug ) ]
  enum MirrorPlane
  {
    XY,
    YZ,
    XZ
  }

  /// Animation modifier that can mirror animations along plane ( XY, YZ, XZ )
  #[ derive( Clone, Copy, Debug ) ]
  pub struct Mirror;

  impl Mirror
  {
    /// Mirror sequencer along plane ( XY, YZ, XZ )
    pub fn along_plane( animation : &Sequencer, plane : MirrorPlane ) -> Sequencer
    {
      let mut animation = animation.clone();

      match plane
      {
        MirrorPlane::XY =>
        {
          for key in animation.keys()
          {
            if key.ends_with( TRANSLATION_PREFIX )
            {
              if let Some( mut player ) = animation.get_mut::< Sequence< Tween< F32x3 > > >( key.into() )
              {

              }
            }
            else if key.ends_with( ROTATION_PREFIX )
            {
              if let Some( mut player ) = animation.get_mut::< Sequence< Tween< F32x3 > > >( key.into() )
              {

              }
            }
            else if key.ends_with( SCALE_PREFIX )
            {
              if let Some( mut player ) = animation.get_mut::< Sequence< Tween< F32x3 > > >( key.into() )
              {

              }
            }
          }
        },
        MirrorPlane::YZ =>
        {

        },
        MirrorPlane::XZ =>
        {

        }
      }

      animation
    }
  }


}

crate::mod_interface!
{
  orphan use
  {
    Mirror,
    MirrorPlane
  };
}

mod private
{
  use animation::
  {
    Tween,
    Sequence,
    Sequencer
  };
  use mingl as gl;
  use gl::{ F32x3, QuatF32 };
  use crate::webgl::animation::base::{ ROTATION_PREFIX, TRANSLATION_PREFIX };

  /// Defines mirror plane
  #[ derive( Clone, Copy, Debug ) ]
  pub enum MirrorPlane
  {
    /// Mirror along XY plane
    XY,
    /// Mirror along YZ plane
    YZ,
    /// Mirror along XZ plane
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
              if let Some( sequence ) = animation.get_mut::< Sequence< Tween< F32x3 > > >( &key )
              {
                for player in sequence.players_mut()
                {
                  player.start_value.0[ 2 ] = - player.start_value.z();
                  player.end_value.0[ 2 ] = - player.end_value.z();
                }
              }
            }
            else if key.ends_with( ROTATION_PREFIX )
            {
              if let Some( sequence ) = animation.get_mut::< Sequence< Tween< QuatF32 > > >( &key )
              {
                for player in sequence.players_mut()
                {
                  player.start_value.0[ 0 ] = - player.start_value.x();
                  player.start_value.0[ 1 ] = - player.start_value.y();
                  player.end_value.0[ 0 ] = - player.end_value.x();
                  player.end_value.0[ 1 ] = - player.end_value.y();

                  // let [ x, y, z ] = player.start_value.to_euler_xyz().0;
                  // player.start_value = QuatF32::from_euler_xyz( [ x, y, z ] );

                  // let [ x, y, z ] = player.end_value.to_euler_xyz().0;
                  // player.end_value = QuatF32::from_euler_xyz( [ x, y + 180_f32.to_radians(), z ] );
                }
              }
            }
          }
        },
        MirrorPlane::YZ =>
        {
          for key in animation.keys()
          {
            if key.ends_with( TRANSLATION_PREFIX )
            {
              if let Some( sequence ) = animation.get_mut::< Sequence< Tween< F32x3 > > >( &key )
              {
                for player in sequence.players_mut()
                {
                  player.start_value.0[ 0 ] = - player.start_value.x();
                  player.end_value.0[ 0 ] = - player.end_value.x();
                }
              }
            }
            else if key.ends_with( ROTATION_PREFIX )
            {
              if let Some( sequence ) = animation.get_mut::< Sequence< Tween< QuatF32 > > >( &key )
              {
                for player in sequence.players_mut()
                {
                  player.start_value.0[ 1 ] = - player.start_value.y();
                  player.start_value.0[ 2 ] = - player.start_value.z();
                  player.end_value.0[ 1 ] = - player.end_value.y();
                  player.end_value.0[ 2 ] = - player.end_value.z();
                }
              }
            }
          }
        },
        MirrorPlane::XZ =>
        {
          for key in animation.keys()
          {
            if key.ends_with( TRANSLATION_PREFIX )
            {
              if let Some( sequence ) = animation.get_mut::< Sequence< Tween< F32x3 > > >( &key )
              {
                for player in sequence.players_mut()
                {
                  player.start_value.0[ 1 ] = - player.start_value.y();
                  player.end_value.0[ 1 ] = - player.end_value.y();
                }
              }
            }
            else if key.ends_with( ROTATION_PREFIX )
            {
              if let Some( sequence ) = animation.get_mut::< Sequence< Tween< QuatF32 > > >( &key )
              {
                for player in sequence.players_mut()
                {
                  player.start_value.0[ 0 ] = - player.start_value.x();
                  player.start_value.0[ 2 ] = - player.start_value.z();
                  player.end_value.0[ 0 ] = - player.end_value.x();
                  player.end_value.0[ 2 ] = - player.end_value.z();
                }
              }
            }
          }
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

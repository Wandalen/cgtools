mod private
{

  use crate::*;

  #[ derive( Default, Debug, Clone, Copy, PartialEq, PartialOrd ) ]
  pub enum Cap
  {
    #[ default ]
    Butt,
    Round( usize ),
    Square
  }

  impl Cap 
  {
    pub fn geometry( &self ) -> ( Vec< f32 >, usize )
    {
      match self 
      {
        Self::Round( segments ) => 
        {
          let g = helpers::round_geometry( *segments );
          let len = g.len();
          ( g.into_iter().flatten().collect(), len )
        },
        Self::Square =>
        {
          let g = helpers::BODY_GEOMETRY;
          let len = g.len();
          ( g.into_iter().flatten().collect(), len )
        },
        Self::Butt => 
        {
          ( Vec::new(), 0 )
        }
      }
    }
  }

}

crate::mod_interface!
{  
  exposed use
  {
    Cap
  };
}
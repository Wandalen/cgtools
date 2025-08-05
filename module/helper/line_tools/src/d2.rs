mod private
{
  
  pub const BODY_VERTEX_SHADER : &'static str = include_str!( "./d2/shaders/body.vert" );
  pub const BODY_MERGED_VERTEX_SHADER : &'static str = include_str!( "./d2/shaders/merged/body.vert" );

  pub const JOIN_ROUND_VERTEX_SHADER : &'static str = include_str!( "./d2/shaders/round_join.vert" );
  pub const JOIN_MITER_VERTEX_SHADER : &'static str = include_str!( "./d2/shaders/miter_join.vert" );
  pub const JOIN_BEVEL_VERTEX_SHADER : &'static str = include_str!( "./d2/shaders/bevel_join.vert" );

  pub const CAP_ROUND_VERTEX_SHADER : &'static str = include_str!( "./d2/shaders/round_cap.vert" );
  pub const CAP_SQUARE_VERTEX_SHADER : &'static str = include_str!( "./d2/shaders/square_cap.vert" );
  pub const CAP_BUTT_VERTEX_SHADER : &'static str = include_str!( "./d2/shaders/empty.vert" );


}

crate::mod_interface!
{
  layer line;
  layer line_merged;

  own use
  {
    BODY_VERTEX_SHADER,
    JOIN_BEVEL_VERTEX_SHADER,
    JOIN_MITER_VERTEX_SHADER,
    JOIN_ROUND_VERTEX_SHADER,
    CAP_BUTT_VERTEX_SHADER,
    CAP_ROUND_VERTEX_SHADER,
    CAP_SQUARE_VERTEX_SHADER,

    BODY_MERGED_VERTEX_SHADER
  };
}

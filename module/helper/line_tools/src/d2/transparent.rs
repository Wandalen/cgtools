mod private
{
  /// The vertex shader for the main body of a line segment.
  pub const BODY_VERTEX_SHADER : &'static str = include_str!( "./transparent/shaders/body.vert" );
   /// The vertex shader for the terminal ends of a line segment.
  pub const BODY_TERMINAL_VERTEX_SHADER : &'static str = include_str!( "./transparent/shaders/body_terminal.vert" );

  /// The vertex shader for rendering a round join between line segments.
  pub const JOIN_ROUND_VERTEX_SHADER : &'static str = include_str!( "./transparent/shaders/join_round.vert" );
  /// The vertex shader for rendering a mitered (sharp) join between line segments.
  pub const JOIN_MITER_VERTEX_SHADER : &'static str = include_str!( "./transparent/shaders/join_miter.vert" );
  /// The vertex shader for rendering a beveled (flat) join between line segments.
  pub const JOIN_BEVEL_VERTEX_SHADER : &'static str = include_str!( "./transparent/shaders/join_bevel.vert" );

  /// The vertex shader for rendering a round cap at the end of a line.
  pub const CAP_ROUND_VERTEX_SHADER : &'static str = include_str!( "./transparent/shaders/cap_round.vert" );
  /// The vertex shader for rendering a square cap at the end of a line.
  pub const CAP_SQUARE_VERTEX_SHADER : &'static str = include_str!( "./transparent/shaders/cap_square.vert" );
   /// The vertex shader for rendering a butt cap, which is a flat termination without any extension.
  pub const CAP_BUTT_VERTEX_SHADER : &'static str = include_str!( "./transparent/shaders/empty.vert" );
}

crate::mod_interface!
{

  /// Layer for line-related functionalities.
  layer line;

  /// A layer dedicated to line join styles (e.g., miter, bevel, round).
  layer join;

  /// A layer dedicated to line cap styles (e.g., butt, round, square).
  layer caps;

  #[ cfg( all( feature = "transparent", not( feature = "solid" ), not( feature = "uv" ) ) ) ]
  exposed use
  {
    Join,
    Cap,
  };

  #[ cfg( all( feature = "transparent", not( feature = "solid" ), not( feature = "uv" ) ) ) ]
  orphan use
  {
    Line
  };

  own use
  {
    BODY_VERTEX_SHADER,
    BODY_TERMINAL_VERTEX_SHADER,
    JOIN_BEVEL_VERTEX_SHADER,
    JOIN_MITER_VERTEX_SHADER,
    JOIN_ROUND_VERTEX_SHADER,
    CAP_BUTT_VERTEX_SHADER,
    CAP_ROUND_VERTEX_SHADER,
    CAP_SQUARE_VERTEX_SHADER,
  };
}

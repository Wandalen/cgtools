//! Rendering command definitions and core primitives.
//!
//! This module contains all the command types that define what can be rendered.
//! All commands are POD types (Copy, Clone, Serialize) as required by FR-A5.

#[ cfg( feature = "enabled" ) ]
mod private
{

  // Allow certain clippy warnings for POD data structures
  #![ allow( clippy::exhaustive_structs ) ]
  #![ allow( clippy::needless_return ) ]
  #![ allow( clippy::cast_possible_truncation ) ]
  #![ allow( clippy::missing_inline_in_public_items ) ]
  #![ allow( clippy::implicit_return ) ]

  use serde::{ Serialize, Deserialize };

  /// Defines stroke appearance for line-based primitives.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq ) ]
  pub struct StrokeStyle
  {
    /// Line width in pixels.
    pub width : f32,
    /// RGBA color values from 0.0 to 1.0.
    pub color : [ f32; 4 ],
    /// Line cap style.
    pub cap_style : LineCap,
    /// Line join style.
    pub join_style : LineJoin,
  }

  /// Line cap style options.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq ) ]
  #[ non_exhaustive ]
  pub enum LineCap
  {
    /// Square cap at line ends.
    Butt,
    /// Rounded cap at line ends.
    Round,
    /// Square cap extending beyond line ends.
    Square,
  }

  /// Line join style options.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq ) ]
  #[ non_exhaustive ]
  pub enum LineJoin
  {
    /// Sharp corner joins.
    Miter,
    /// Rounded corner joins.
    Round,
    /// Beveled corner joins.
    Bevel,
  }

  /// Text anchor positioning options.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq ) ]
  #[ non_exhaustive ]
  pub enum TextAnchor
  {
    /// Anchor at top-left corner.
    TopLeft,
    /// Anchor at top-center.
    TopCenter,
    /// Anchor at top-right corner.
    TopRight,
    /// Anchor at center-left.
    CenterLeft,
    /// Anchor at center.
    Center,
    /// Anchor at center-right.
    CenterRight,
    /// Anchor at bottom-left corner.
    BottomLeft,
    /// Anchor at bottom-center.
    BottomCenter,
    /// Anchor at bottom-right corner.
    BottomRight,
  }

  /// Font style definition for text rendering.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq ) ]
  pub struct FontStyle
  {
    /// Font size in pixels.
    pub size : f32,
    /// RGBA color values from 0.0 to 1.0.
    pub color : [ f32; 4 ],
    /// Font weight (100-900, normal is 400).
    pub weight : u16,
    /// Whether text is italic.
    pub italic : bool,
    /// Font family identifier (index into font registry).
    pub family_id : u32,
  }

  /// 2D position coordinate.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq ) ]
  pub struct Point2D
  {
    /// X coordinate.
    pub x : f32,
    /// Y coordinate.
    pub y : f32,
  }

  /// Represents a 2D transformation with position, rotation, and scale.
  #[ derive( Debug, Clone, Copy, PartialEq, Serialize, Deserialize ) ]
  pub struct Transform2D
  {
    /// The translation component, as `[x, y]`.
    pub position : [ f32; 2 ],
    /// The rotation component, in radians.
    pub rotation : f32,
    /// The scale component, as `[x, y]`.
    pub scale : [ f32; 2 ]
  }

  impl Transform2D
  {
    /// Creates a new `Transform2D` from its components.
    pub fn new< V1, V2 >( position : V2, rotation : V1, scale : V2 ) -> Self
    where
      V1 : Into< f32 >,
      V2 : Into< [ f32; 2 ] >
    {
      Self
      {
        position : position.into(),
        rotation : rotation.into(),
        scale : scale.into()
      }
    }

    /// Sets the position of the transform.
    pub fn position_set< V2 >( &mut self, position : V2 )
    where
      V2 : Into< [ f32; 2 ] >
    {
      self.position = position.into();
    }

    /// Sets the rotation of the transform.
    pub fn rotation_set< V1 >( &mut self, rotation : V1 )
    where
      V1 : Into< f32 >
    {
      self.rotation = rotation.into();
    }

    /// Sets the scale of the transform.
    pub fn scale_set< V2 >( &mut self, scale : V2 )
    where
      V2 : Into< [ f32; 2 ] >
    {
      self.scale = scale.into();
    }
  }

  impl Default for Transform2D
  {
    fn default() -> Self
    {
      Self { position : Default::default(), rotation : Default::default(), scale : [ 1.0; 2 ] }
    }
  }

  /// Specifies the rendering mode for 2D geometry.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
  pub enum GeometryMode
  {
    /// Renders the geometry as a series of filled triangles.
    Triangles,
    /// Renders the geometry as a series of lines.
    Lines,
  }

  /// A command to render a piece of 2D geometry.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq ) ]
  pub struct Geometry2DCommand
  {
    /// The unique identifier of the geometry resource to be drawn.
    pub id : u32,
    /// The 2D transformation to apply to the geometry.
    pub transform : Transform2D,
    /// The solid color to apply to the geometry, as `[r, g, b]`.
    pub color : [ f32; 3 ],
    /// The mode (triangles or lines) to use for rendering.
    pub mode : GeometryMode
  }

  /// A command to render a sprite.
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq ) ]
  pub struct SpriteCommand
  {
    /// The unique identifier of the texture resource to be drawn.
    pub id : u32,
    /// The 2D transformation to apply to the sprite.
    pub transform : Transform2D,
  }

  /// Line rendering command (FR-B1).
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq ) ]
  pub struct LineCommand
  {
    /// Starting point of the line.
    pub start : Point2D,
    /// Ending point of the line.
    pub end : Point2D,
    /// Stroke style for the line.
    pub style : StrokeStyle,
  }

  /// Bezier curve rendering command (FR-B2).
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq ) ]
  pub struct CurveCommand
  {
    /// Starting point of the curve.
    pub start : Point2D,
    /// First control point.
    pub control1 : Point2D,
    /// Second control point.
    pub control2 : Point2D,
    /// Ending point of the curve.
    pub end : Point2D,
    /// Stroke style for the curve.
    pub style : StrokeStyle,
  }

  /// Text rendering command (FR-B3).
  #[ derive( Debug, Clone, Copy, PartialEq ) ]
  pub struct TextCommand
  {
    /// Position for text anchor.
    pub position : Point2D,
    /// Text content (limited to 64 characters for POD and serde compliance).
    pub text : [ u8; 64 ],
    /// Actual length of text content.
    pub text_len : u8,
    /// Font and style information.
    pub font_style : FontStyle,
    /// Text anchor position.
    pub anchor : TextAnchor,
  }

  // Manual Serialize/Deserialize implementation for TextCommand
  impl Serialize for TextCommand
  {
    fn serialize< S >( &self, serializer: S ) -> Result< S::Ok, S::Error >
    where
      S: serde::Serializer,
    {
      use serde::ser::SerializeStruct;
      let mut state = serializer.serialize_struct( "TextCommand", 5 )?;
      state.serialize_field( "position", &self.position )?;
      state.serialize_field( "text", &self.text[ ..self.text_len as usize ] )?;
      state.serialize_field( "text_len", &self.text_len )?;
      state.serialize_field( "font_style", &self.font_style )?;
      state.serialize_field( "anchor", &self.anchor )?;
      state.end()
    }
  }

  impl< 'de > serde::Deserialize< 'de > for TextCommand
  {
    fn deserialize< D >( deserializer: D ) -> Result< Self, D::Error >
    where
      D: serde::Deserializer< 'de >,
    {
      use serde::de::{ self, MapAccess, Visitor };

      struct TextCommandVisitor;

      impl< 'de > Visitor< 'de > for TextCommandVisitor
      {
        type Value = TextCommand;

        fn expecting( &self, formatter: &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
        {
          formatter.write_str( "struct TextCommand" )
        }

        fn visit_map< V >( self, mut map: V ) -> Result< TextCommand, V::Error >
        where
          V: MapAccess< 'de >,
        {
          let mut position = None;
          let mut text_data: Option< Vec< u8 > > = None;
          let mut text_len = None;
          let mut font_style = None;
          let mut anchor = None;

          while let Some( key ) = map.next_key()?
          {
            match key
            {
              "position" => position = Some( map.next_value()? ),
              "text" => text_data = Some( map.next_value()? ),
              "text_len" => text_len = Some( map.next_value()? ),
              "font_style" => font_style = Some( map.next_value()? ),
              "anchor" => anchor = Some( map.next_value()? ),
              _ => { let _: serde::de::IgnoredAny = map.next_value()?; }
            }
          }

          let position = position.ok_or_else( || de::Error::missing_field( "position" ) )?;
          let text_data = text_data.ok_or_else( || de::Error::missing_field( "text" ) )?;
          let text_len = text_len.ok_or_else( || de::Error::missing_field( "text_len" ) )?;
          let font_style = font_style.ok_or_else( || de::Error::missing_field( "font_style" ) )?;
          let anchor = anchor.ok_or_else( || de::Error::missing_field( "anchor" ) )?;

          let mut text = [ 0u8; 64 ];
          let actual_len = text_data.len().min( 64 );
          text[ ..actual_len ].copy_from_slice( &text_data[ ..actual_len ] );

          Ok( TextCommand { position, text, text_len, font_style, anchor } )
        }
      }

      const FIELDS: &[ &str ] = &[ "position", "text", "text_len", "font_style", "anchor" ];
      deserializer.deserialize_struct( "TextCommand", FIELDS, TextCommandVisitor )
    }
  }

  /// Tilemap rendering command (FR-B4).
  #[ derive( Debug, Clone, Copy, PartialEq ) ]
  pub struct TilemapCommand
  {
    /// Top-left position of tilemap.
    pub position : Point2D,
    /// Width of each tile in pixels.
    pub tile_width : f32,
    /// Height of each tile in pixels.
    pub tile_height : f32,
    /// Number of tiles horizontally.
    pub map_width : u32,
    /// Number of tiles vertically.
    pub map_height : u32,
    /// Tileset texture identifier.
    pub tileset_id : u32,
    /// Tile indices (limited to 32 for POD and serde compliance).
    pub tile_data : [ u16; 32 ],
    /// Actual number of tiles used.
    pub tile_count : u32,
  }

  // Manual Serialize/Deserialize implementation for TilemapCommand
  impl Serialize for TilemapCommand
  {
    fn serialize< S >( &self, serializer: S ) -> Result< S::Ok, S::Error >
    where
      S: serde::Serializer,
    {
      use serde::ser::SerializeStruct;
      let mut state = serializer.serialize_struct( "TilemapCommand", 8 )?;
      state.serialize_field( "position", &self.position )?;
      state.serialize_field( "tile_width", &self.tile_width )?;
      state.serialize_field( "tile_height", &self.tile_height )?;
      state.serialize_field( "map_width", &self.map_width )?;
      state.serialize_field( "map_height", &self.map_height )?;
      state.serialize_field( "tileset_id", &self.tileset_id )?;
      state.serialize_field( "tile_data", &self.tile_data[ ..self.tile_count as usize ] )?;
      state.serialize_field( "tile_count", &self.tile_count )?;
      state.end()
    }
  }

  impl< 'de > serde::Deserialize< 'de > for TilemapCommand
  {
    fn deserialize< D >( deserializer: D ) -> Result< Self, D::Error >
    where
      D: serde::Deserializer< 'de >,
    {
      use serde::de::{ self, MapAccess, Visitor };

      struct TilemapCommandVisitor;

      impl< 'de > Visitor< 'de > for TilemapCommandVisitor
      {
        type Value = TilemapCommand;

        fn expecting( &self, formatter: &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
        {
          formatter.write_str( "struct TilemapCommand" )
        }

        fn visit_map< V >( self, mut map: V ) -> Result< TilemapCommand, V::Error >
        where
          V: MapAccess< 'de >,
        {
          let mut position = None;
          let mut tile_width = None;
          let mut tile_height = None;
          let mut map_width = None;
          let mut map_height = None;
          let mut tileset_id = None;
          let mut tile_data_vec: Option< Vec< u16 > > = None;
          let mut tile_count = None;

          while let Some( key ) = map.next_key()?
          {
            match key
            {
              "position" => position = Some( map.next_value()? ),
              "tile_width" => tile_width = Some( map.next_value()? ),
              "tile_height" => tile_height = Some( map.next_value()? ),
              "map_width" => map_width = Some( map.next_value()? ),
              "map_height" => map_height = Some( map.next_value()? ),
              "tileset_id" => tileset_id = Some( map.next_value()? ),
              "tile_data" => tile_data_vec = Some( map.next_value()? ),
              "tile_count" => tile_count = Some( map.next_value()? ),
              _ => { let _: serde::de::IgnoredAny = map.next_value()?; }
            }
          }

          let position = position.ok_or_else( || de::Error::missing_field( "position" ) )?;
          let tile_width = tile_width.ok_or_else( || de::Error::missing_field( "tile_width" ) )?;
          let tile_height = tile_height.ok_or_else( || de::Error::missing_field( "tile_height" ) )?;
          let map_width = map_width.ok_or_else( || de::Error::missing_field( "map_width" ) )?;
          let map_height = map_height.ok_or_else( || de::Error::missing_field( "map_height" ) )?;
          let tileset_id = tileset_id.ok_or_else( || de::Error::missing_field( "tileset_id" ) )?;
          let tile_data_vec = tile_data_vec.ok_or_else( || de::Error::missing_field( "tile_data" ) )?;
          let tile_count = tile_count.ok_or_else( || de::Error::missing_field( "tile_count" ) )?;

          let mut tile_data = [ 0u16; 32 ];
          let actual_len = tile_data_vec.len().min( 32 );
          tile_data[ ..actual_len ].copy_from_slice( &tile_data_vec[ ..actual_len ] );

          Ok( TilemapCommand { position, tile_width, tile_height, map_width, map_height, tileset_id, tile_data, tile_count } )
        }
      }

      const FIELDS: &[ &str ] = &[ "position", "tile_width", "tile_height", "map_width", "map_height", "tileset_id", "tile_data", "tile_count" ];
      deserializer.deserialize_struct( "TilemapCommand", FIELDS, TilemapCommandVisitor )
    }
  }

  /// Particle emitter rendering command (FR-B5).
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq ) ]
  pub struct ParticleEmitterCommand
  {
    /// Emitter position.
    pub position : Point2D,
    /// Emission rate (particles per second).
    pub emission_rate : f32,
    /// Particle lifetime in seconds.
    pub particle_lifetime : f32,
    /// Initial particle velocity.
    pub initial_velocity : Point2D,
    /// Velocity variance.
    pub velocity_variance : Point2D,
    /// Particle size in pixels.
    pub particle_size : f32,
    /// Size variance.
    pub size_variance : f32,
    /// Particle color.
    pub particle_color : [ f32; 4 ],
    /// Color variance.
    pub color_variance : [ f32; 4 ],
  }

  /// Main render command enum wrapping all primitives (FR-A4).
  #[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq ) ]
  #[ non_exhaustive ]
  pub enum RenderCommand
  {
    /// Line primitive.
    Line( LineCommand ),
    /// Bezier curve primitive.
    Curve( CurveCommand ),
    /// Text primitive.
    Text( TextCommand ),
    /// Tilemap primitive.
    Tilemap( TilemapCommand ),
    /// Particle emitter primitive.
    ParticleEmitter( ParticleEmitterCommand ),
    /// 2D geometry primitive.
    Geometry2DCommand( Geometry2DCommand ),
    /// Sprite primitive.
    SpriteCommand( SpriteCommand )
  }

  impl Default for StrokeStyle
  {
    fn default() -> Self
    {
      Self
      {
        width: 1.0,
        color: [ 0.0, 0.0, 0.0, 1.0 ], // Black
        cap_style: LineCap::Butt,
        join_style: LineJoin::Miter,
      }
    }
  }

  impl Default for FontStyle
  {
    fn default() -> Self
    {
      Self
      {
        size: 12.0,
        color: [ 0.0, 0.0, 0.0, 1.0 ], // Black
        weight: 400, // Normal
        italic: false,
        family_id: 0,
      }
    }
  }

  impl Default for Point2D
  {
    fn default() -> Self
    {
      Self { x: 0.0, y: 0.0 }
    }
  }

  impl Point2D
  {
    /// Creates a new 2D point.
    #[ must_use ]
    #[ inline ]
    pub const fn new( x: f32, y: f32 ) -> Self
    {
      Self { x, y }
    }
  }

  impl TextCommand
  {
    /// Creates a new text command with the given text content.
    /// Text longer than 63 characters will be truncated.
    #[ must_use ]
    #[ inline ]
    pub fn new( text: &str, position: Point2D, font_style: FontStyle, anchor: TextAnchor ) -> Self
    {
      let mut text_bytes = [ 0u8; 64 ];
      let text_len = text.len().min( 63 );
      text_bytes[ ..text_len ].copy_from_slice( &text.as_bytes()[ ..text_len ] );

      return Self
      {
        position,
        text: text_bytes,
        text_len: text_len as u8,
        font_style,
        anchor,
      }
    }

    /// Extracts the text content as a string slice.
    #[ must_use ]
    #[ inline ]
    pub fn text( &self ) -> &str
    {
      return core::str::from_utf8( &self.text[ ..self.text_len as usize ] )
        .unwrap_or( "" )
    }
  }

  impl TilemapCommand
  {
    /// Creates a new tilemap command with the given tile data.
    /// Tile data longer than 32 tiles will be truncated.
    #[ must_use ]
    #[ inline ]
    #[ allow( clippy::cast_possible_truncation ) ]
    pub fn new(
      position: Point2D,
      tile_width: f32,
      tile_height: f32,
      map_width: u32,
      map_height: u32,
      tileset_id: u32,
      tiles: &[ u16 ]
    ) -> Self
    {
      let mut tile_data = [ 0u16; 32 ];
      let tile_count = ( tiles.len() as u32 ).min( 32 );
      tile_data[ ..tile_count as usize ].copy_from_slice( &tiles[ ..tile_count as usize ] );

      return Self
      {
        position,
        tile_width,
        tile_height,
        map_width,
        map_height,
        tileset_id,
        tile_data,
        tile_count,
      }
    }

    /// Returns the tile data as a slice.
    #[ must_use ]
    #[ inline ]
    pub fn tiles( &self ) -> &[ u16 ]
    {
      return &self.tile_data[ ..self.tile_count as usize ]
    }
  }
}

#[ cfg( feature = "enabled" ) ]
pub use private::*;

use std::rc::Rc;
use nutype::nutype;
use serde::{ Deserialize, Serialize };

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct MapFile
{
  pub metadata : Metadata,
  pub map : crate::MapSerde,
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Metadata
{
  config : Config,
  bounds : ( [ i32; 2 ], [ i32; 2 ] ),
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Scheme
{
  tile_variants : Vec< TileName >,
}

impl Scheme
{
  pub fn tile_name( &self, id : TileId ) -> TileName
  {
    self.tile_variants[ id.into_inner() ].clone()
  }
}

#[ nutype( derive( Debug, PartialEq, AsRef, Deref, Clone ) ) ]
pub struct TileName( Rc< str > );

impl Serialize for TileName
{
  fn serialize< S >( &self, serializer : S ) -> Result< S::Ok, S::Error >
  where
    S : serde::Serializer
  {
    serializer.collect_str( self.as_ref() )
  }
}

impl< 'de > Deserialize< 'de > for TileName
{
  fn deserialize< D >( deserializer : D ) -> Result< Self, D::Error >
  where
    D : serde::Deserializer< 'de >
  {
    let s = String::deserialize( deserializer )?;
    Ok( TileName::new( s.into() ) )
  }
}

#[ nutype( derive( Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord ) ) ]
pub struct TileId( usize );

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Config
{
  pub player_colors : Vec< [ u8; 3 ] >,
  pub tiles_config : Vec< TileConfig >,
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct TileConfig
{
  pub name : TileName,
  pub attributes : Attributes,
  pub sprite : Sprite
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Sprite
{
  pub source : Box< str >,
  pub width : u32,
  pub height : u32,
  pub x_offset : u32,
  pub y_offset : u32,
  pub scale : f32,
}

#[ derive( Debug, Serialize, Deserialize, PartialEq, Default ) ]
pub struct Attributes
{
  #[ serde ( default, skip_serializing_if = "is_default_bool" ) ]
  pub is_static : bool,
  #[ serde ( default, skip_serializing_if = "is_default_bool" ) ]
  pub is_constructable : bool,
  #[ serde ( default, skip_serializing_if = "is_default_u32" ) ]
  pub areal : u32,
  #[ serde ( default, skip_serializing_if = "is_default_u32" ) ]
  pub cost : u32,
  #[ serde ( default, skip_serializing_if = "is_default_u32" ) ]
  pub wage : u32,
  #[ serde ( default, skip_serializing_if = "is_default_u32" ) ]
  pub strength : u32,
}

fn is_default_bool( value : &bool ) -> bool
{
  *value == false
}

fn is_default_u32( value : &u32 ) -> bool
{
  *value == 0
}

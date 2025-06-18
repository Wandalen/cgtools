use crate::{ triaxial::TriAxial, Axial };
use std::{ cell::RefCell, rc::Rc };
use serde::{ Deserialize, Serialize };

type HashMap< K, V > = rustc_hash::FxHashMap< K, V >;
type HashSet< V > = rustc_hash::FxHashSet< V >;

pub fn read_map_file( json : &str ) -> serde_json::Result< Instance >
{
  let mut str_pool = StrPool::default();
  let MapFile { config, map, .. } = serde_json::from_str::< MapFile >( json )?;
  let ( object_config, terrain_config, player_colors ) = read_config( config, &mut str_pool );
  let ( tile_map, river_map ) = read_map( map, &mut str_pool );

  Ok
  (
    Instance
    {
      object_config,
      terrain_config,
      tile_map,
      river_map,
      player_colors,
    }
  )
}

fn read_config
(
  ConfigSerde { player_colors, object_config, terrain_config } : ConfigSerde,
  str_pool : &mut StrPool
)
-> ( HashMap< Rc< str >, Properties >, HashMap< Rc< str >, Properties >, Vec< minwebgl::F32x3 > )
{
  let objects = read_properties( object_config, str_pool );
  let terrain = read_properties( terrain_config, str_pool );
  let player_colors = player_colors.into_iter()
  .map( | [ r, g, b ] | minwebgl::F32x3::new( r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0 ) )
  .collect();

  ( objects, terrain, player_colors )
}

fn read_properties
(
  properties : Vec< PropertiesSerde >,
  str_pool : &mut StrPool,
)
-> HashMap::< Rc< str >, Properties >
{
  let mut ret = HashMap::default();

  for prop in properties
  {
    let mut attributes = HashMap::default();
    for ( name, attribute ) in prop.attributes
    {
      let name = str_pool.get( &name );
      attributes.insert( name, attribute );
    }

    let properties = Properties { attributes, sprite : prop.sprite };

    // if let Some( sprite ) = &object.sprite
    // {
    //   let sprite_source = str_pool.get( &sprite.source );
    //   if !texture_map.contains_key( &sprite_source )
    //   {
    //     let ( texture, size ) = crate::helper::load_texture( gl, document, &sprite_source );
    //     let texture = Texture { size, texture };
    //     texture_map.insert( sprite_source.clone(), texture );
    //   }
    // }

    let name = str_pool.get( &prop.name );
    ret.insert( name, properties );
  }
  ret
}

fn read_map( map_serde : MapSerde, str_pool : &mut StrPool )
-> ( HashMap< Axial, Tile >, HashSet< TriAxial > )
{
  let mut tile_map = HashMap::default();
  for TileSerde { coord, player_index, object_name, terrain_name } in map_serde.tile_map
  {
    let terrain_name = str_pool.get( &terrain_name );
    let object_name = object_name.map( | name | str_pool.get( &name ) );
    let tile = Tile
    {
      terrain_type : terrain_name,
      object_type : object_name,
      player_id : player_index,
      coord,
    };
    tile_map.insert( coord, tile );
  }

  let river_map = HashSet::from_iter( map_serde.river_map.into_iter() );

  ( tile_map, river_map )
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct MapFile
{
  pub metadata : MetadataSerde,
  pub config : ConfigSerde,
  pub map : MapSerde,
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct MetadataSerde
{
  pub bounds : [ Axial; 2 ],
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct ConfigSerde
{
  pub player_colors : Vec< [ u8; 3 ] >,
  pub object_config : Vec< PropertiesSerde >,
  pub terrain_config : Vec< PropertiesSerde >,
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct PropertiesSerde
{
  pub name : String,
  pub attributes : serde_json::Map< String, serde_json::Value >,
  pub sprite : Option< Sprite >
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct MapSerde
{
  pub tile_map : Vec< TileSerde >,
  pub river_map : Vec< TriAxial >,
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct TileSerde
{
  pub coord : Axial,
  pub player_index : u32,
  pub object_name : Option< String >,
  pub terrain_name : String,
}

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Sprite
{
  pub source : String,
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
  pub area : u32,
  #[ serde ( default, skip_serializing_if = "is_default_u32" ) ]
  pub cost : u32,
  #[ serde ( default, skip_serializing_if = "is_default_u32" ) ]
  pub wage : u32,
  #[ serde ( default, skip_serializing_if = "is_default_u32" ) ]
  pub strength : u32,
  #[ serde ( default, skip_serializing_if = "is_default_bool" ) ]
  pub grows_in_random_direction : bool,
  #[ serde ( default, skip_serializing_if = "is_default_bool" ) ]
  pub appears_on_dead_body : bool,
  #[ serde ( default, skip_serializing_if = "is_default_bool" ) ]
  pub can_exist_only_near_sea : bool,
  #[ serde ( default, skip_serializing_if = "is_default_u32" ) ]
  pub income : u32,
  #[ serde ( default, skip_serializing_if = "is_default_bool" ) ]
  pub is_terrain : bool,
}

fn is_default_bool( value : &bool ) -> bool
{
  *value == false
}

fn is_default_u32( value : &u32 ) -> bool
{
  *value == 0
}

#[ derive( Debug, Default, Clone ) ]
pub struct StrPool( HashSet< Rc< str > > );

impl StrPool
{
  pub fn get( &mut self, val : &str ) -> Rc< str >
  {
    match self.0.get( val )
    {
      Some( val ) => val.clone(),
      None =>
      {
        let val : Rc< str > = val.into();
        self.0.insert( val.clone() );
        val
      }
    }
  }
}

#[ derive( Debug ) ]
pub struct Instance
{
  pub object_config : HashMap< Rc< str >, Properties >,
  pub terrain_config : HashMap< Rc< str >, Properties >,
  pub tile_map : HashMap< Axial, Tile >,
  pub river_map : HashSet< TriAxial >,
  pub player_colors : Vec< minwebgl::F32x3 >,
}

#[ derive( Debug, Clone ) ]
pub struct Tile
{
  terrain_type : Rc< str >,
  object_type : Option< Rc< str > >,
  player_id : u32,
  coord : Axial,
}

#[ derive( Debug) ]
pub struct Properties
{
  pub attributes : HashMap< Rc< str >, serde_json::Value >,
  pub sprite : Option< Sprite >,
}

pub struct Texture
{
  pub size : Rc< RefCell< minwebgl::U32x2 > >,
  pub texture : Option< web_sys::WebGlTexture >
}

use std::collections::{ HashMap, HashSet };
use std::hash::Hash;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand::prelude::SliceRandom;
use rayon::prelude::*;
use serde::{ Serialize, Deserialize };
use std::sync::Arc;
use std::ops::Range;
use rand::Rng;
use web_sys::console;
use minwebgl::JsValue;

/// Used for evaluating neighbour tiles coords
/// and for indexing posible neighbour tiles in [`Relations`]
#[
  derive
  (
    Debug, Clone, Copy, Serialize, Deserialize,
    Eq, PartialEq, Ord, PartialOrd, Hash
  )
]
#[ serde( untagged ) ]
enum Direction
{
  /// West
  W,
  /// East
  E,
  /// North
  N,
  /// South
  S,
  /// Up
  U,
  /// Down
  D
}

impl Direction
{
  /// Maps [ `Direction` ] into dimention and [ `Point` ]
  /// position relatively to current [ `Point` ]
  fn difference( &self ) -> ( usize, isize )
  {
    match self
    {
      Direction::W => ( 0, -1 ),
      Direction::E => ( 0, 1 ),
      Direction::N => ( 1, -1 ),
      Direction::S => ( 1, 1 ),
      Direction::U => ( 2, -1 ),
      Direction::D => ( 2, 1 ),
    }
  }
}

/// Store set of possible tile states that can be adjacent
/// to current tile.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( untagged ) ]
enum Relation
{
  /// For [ `Direction` ] independed states. Will be applyed
  /// for every [ `Direction` ] in propagate state
  Isotropic( HashSet< u8 > ),
  /// For [ `Direction` ] depended states. Will be applyed for
  /// certain [ `Direction` ] in propagate state
  Anisotropic( HashMap< Direction, HashSet< u8 > > )
}

/// Store list of posible neighbour tiles
/// states for each state of current tile
#[derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct Relations( Vec< Relation > );

impl Relations
{
  /// Returns all [ `Relation::Isotropic` ] variants from [ `Relations` ] list
  fn get_all_isotropic( &self ) -> Vec< ( u8, Relation ) >
  {
    self.0.iter()
    .cloned()
    .enumerate()
    .filter_map
    (
      | ( i, r ) |
      {
        if let Relation::Isotropic( _ ) = r
        {
          Some( ( i as u8, r.clone() ) )
        }
        else
        {
          None
        }
      }
    )
    .collect::< Vec< _ > >()
  }
}

/// Coordinates of 2d tiles
#[ derive( Hash, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd ) ]
struct Point
{
  x : isize,
  y : isize
}

impl Point
{
  fn new( x : isize, y : isize ) -> Self
  {
    Self
    {
      x,
      y
    }
  }
}

/// General state for wave function collapse algorithm
///
/// Made with builder pattern. User must set with methods
/// size, front, relations fields and then call calculate method
/// that returns map [ `Vec< Vec< u8 > >` ]. Wfc calculate limited size
/// patterns map.
struct Wfc
{
  /// Stores neighbour tiles from other chunk maps
  edges : HashMap< Direction, Vec< u8 > >,
  /// Map that contains current superposition of variants for each tile
  map : Vec< Vec< Vec< u8 > > >,
  /// Current front from which the collapse and propagate steps of
  /// the algorithm will occur. Contains tile coords ( [ `Point` ] ).
  front : Vec< Point >,
  /// Store list of posible neighbour tiles
  /// states for each state of current tile
  relations : Relations,
}

impl Wfc
{
  fn new() -> Self
  {
    Self
    {
      edges : HashMap::new(),
      map : vec![],
      front : vec![],
      relations : Relations( vec![] )
    }
  }

  fn size( mut self, size : ( usize, usize ) ) -> Self
  {
    self.map = vec![ vec![ vec![]; size.0 ]; size.1 ];
    self
  }

  fn edges( mut self, edges : HashMap< Direction, Vec< u8 > > ) -> Self
  {
    self.edges = edges;
    self
  }

  fn set_points( mut self, points_to_set : Vec< ( Point, u8 ) > ) -> Self
  {
    for ( p, value ) in points_to_set
    {
      self.map[ p.y as usize ][ p.x as usize ] = vec![ value ];
    }
    self
  }

  fn front( mut self, front : Vec< Point > ) -> Self
  {
    self.front = front;
    self
  }

  fn relations( mut self, relations : Relations ) -> Self
  {
    self.relations = relations;

    let all_variants = ( 0..( self.relations.0.len() as u8 ) ).collect::< Vec< _ > >();
    if !self.map.is_empty() && !self.map[ 0 ].is_empty()
    {
      self.map = vec![ vec![ all_variants; self.map[0].len() ]; self.map.len() ];
    }
    self
  }

  /// Propagate each tile from list points and return new variants list for each tile
  fn calculate_variants( &self, points : Vec< Point > ) -> Vec<( Point, Vec< u8 > ) >
  {
    let relations = self.relations.clone();
    let directions = self.edges
    .keys()
    .cloned()
    .collect::< Vec< _ > >();
    let mut map = self.map.clone();
    points.into_iter()
    .map
    (
      | p |
      {
        let new_variants = propagate_cell( &map, &directions, &relations, p );
        map[ p.y as usize ][ p.x as usize ] = new_variants.clone();
        ( p, new_variants )
      }
    )
    .collect()
  }

  /// Calculate minimal entropy for tile set in points ( [ `Vec< Point >` ] ).
  /// Then returns tile coordinates only with minimal entropy
  fn get_with_min_entrophy( &self, points : &Vec< Point > ) -> Vec< Point >
  {
    let map = Arc::new( self.map.clone() );
    let iter = points.clone()
    .into_par_iter()
    .map_init
    (
      || map.clone(),
      | m, p |
      {
        ( p, m[ p.y as usize ][ p.x as usize ].len() )
      }
    )
    .filter
    (
      | ( _, v ) | *v > 1
    );
    let min_entropy = iter.clone()
    .map( | ( _, v ) | v )
    .min()
    .unwrap_or( self.relations.0.len() );
    iter
    .filter( | ( _, v ) | *v <= min_entropy )
    .map( | ( p, _ ) | p ).collect::< Vec< _ > >()
  }

  /// Returns points that still have many tile variants (>1)
  fn get_unsolved( &self, points : Vec< Point > ) -> Vec< Point >
  {
    let map = Arc::new( self.map.clone() );
    points.into_par_iter()
    .map_init
    (
      || map.clone(),
      | m, p |
      {
        ( p, m[ p.y as usize ][ p.x as usize ].len() )
      }
    )
    .filter( | ( _, v ) | *v > 1 )
    .map( | ( p, _ ) | p )
    .collect::< Vec< _ > >()
  }

  /// For each tile coordinates with minimal entropy in front
  /// choose any posible variant and write result back to map
  fn collapse( &mut self )
  {
    let front = self.front.clone();
    let map = Arc::new( self.map.clone() );
    let mut r = SmallRng::from_rng( rand::thread_rng() )
    .unwrap();
    let invalid_value = self.relations.0.len() as u8;
    let with_min_entrophy = self.get_with_min_entrophy( &front );
    let collapsed = with_min_entrophy.into_iter()
    .map
    (
      | p |
      {
        (
          p,
          map[ p.y as usize ][ p.x as usize ].choose( &mut r )
          .unwrap_or( &invalid_value )
        )
      }
    )
    .collect::< Vec< _ > >();
    for ( p, v ) in collapsed
    {
      self.map[ p.y as usize ][ p.x as usize ] = vec![ *v ];
    }
  }

  /// Set new front and update front tiles variants
  fn propagate( &mut self )
  {
    if self.map.is_empty() || self.map[ 0 ].is_empty()
    {
      return;
    }
    let front = self.front
    .clone();
    let directions = self.edges
    .keys()
    .cloned()
    .collect::< Vec< _ > >();
    let diffs = directions.into_iter()
    .map( | d | d.difference() )
    .collect::< Vec< _ > >();
    // 1. Split current front on solved and unsolved parts
    let unsolved_front = self.get_unsolved( front.clone() );
    let solved_front = front.iter()
    .collect::< HashSet< _ > >()
    .difference( &unsolved_front.iter().collect::< HashSet< _ > >() )
    .cloned()
    .cloned()
    .collect::< Vec< _ > >();
    // 2. Calculate new front from solved tiles neighbours
    let mut new_front = get_neighbours( solved_front, diffs.clone() );
    // 3. Filter new front tiles outside map
    new_front = new_front.into_iter()
    .filter
    (
      | p |
      {
        p.x >= 0 && p.y >= 0
        && p.x < self.map[ 0 ].len() as isize
        && p.y < self.map.len() as isize
      }
    )
    .collect::< Vec< _ > >();
    // 4. Filter unsolved tiles in new front
    new_front = self.get_unsolved( new_front );
    // 5. Add old unsolved tiles to new front
    new_front.extend( unsolved_front );
    // 6. Update variants for tiles in new front
    let new_variants = self.calculate_variants( new_front.clone() );
    // 7. Set new tiles variants in map.
    for ( p, variants ) in new_variants
    {
      self.map[ p.y as usize ][ p.x as usize ] = variants;
    }
    self.front = new_front;
  }

  /// Do repeatedly cycle collapse-propagate while front isn't empty.
  /// When the cycle ended check and handle errors for each tile and
  /// then returns [ `Vec< Vec< u8 > >` ]
  fn calculate( &mut self ) -> Result< Vec< Vec< u8 > >, String >
  {
    while !self.front.is_empty()
    {
      self.collapse();
      self.propagate();
    }

    let all_variants = ( 0..( self.relations.0.len() as u8 ) ).collect::< Vec< _ > >();
    self.map.par_iter_mut()
    .for_each(
      | row |
      {
        row.iter_mut()
        .for_each
        (
          | v |
          {
            let mut rng = SmallRng::from_rng( rand::thread_rng() ).unwrap();
            if v.is_empty()
            {
              *v = vec![ *all_variants.choose( &mut rng ).unwrap() ]
            }
            else
            {
              *v = vec![ *v.choose( &mut rng ).unwrap() ]
            }
          }
        )
      }
    );

    Ok
    (
      self.map
      .clone()
      .into_par_iter()
      .map
      (
        | row |
        {
          row
          .into_iter()
          .flatten()
          .collect()
        }
      )
      .collect()
    )
  }
}

/// Print front shape on map with 'x'. If show_collapsed true display also collapsed tiles with '#'
fn print_front( map : &Vec< Vec< Vec< u8 > > >, front : &Vec< Point >, show_collapsed : bool )
{
  let mut front_map = vec![ vec![ ' '; map[ 0 ].len() ]; map.len() ];
  for p in front
  {
    front_map[ p.y as usize ][ p.x as usize ] = 'x';
  }
  if show_collapsed
  {
    for ( i, row ) in map.iter().enumerate()
    {
      for ( j, variants ) in row.iter().enumerate()
      {
        if variants.len() <= 1
        {
          front_map[ i ][ j ] = '#';
        }
      }
    }
  }

  let mut map_string = "\n".to_string();
  for row in front_map
  {
    for value in row
    {
      map_string.push( value );
    }
    map_string += "\n";
  }
  console::log_1( &JsValue::from( format!( "{map_string}" ) ) );
}

/// Print map with variants count for each tile
fn print_variants_count( map : &Vec< Vec< Vec< u8 > > > )
{
  let mut map_string = "\n".to_string();
  for row in map
  {
    for value in row
    {
      map_string += &value.len().to_string();
    }
    map_string += "\n";
  }
  console::log_1( &JsValue::from( format!( "{map_string}" ) ) );
}

/// Calculates current tile variants relatively to neighbour tiles
/// and regardless of [ `Relation` ] type
fn propagate_cell
(
  map : &Vec< Vec< Vec< u8 > > >,
  directions : &Vec< Direction >,
  relations : &Relations,
  point : Point
) -> Vec< u8 >
{
  calculate_isotropic_variants( map, directions, relations, point )
}

/// Returns neighbour tile variants
fn get_neighbour_variants
(
  map : &Vec< Vec< Vec< u8 > > >,
  point : Point,
  diff : ( usize, isize )
) -> Vec< u8 >
{
  let ( dim, diff ) = diff;
  let h = map.len();
  let w = map[ 0 ].len();
  let empty = Vec::< u8 >::new();
  match ( dim, diff )
  {
    ( 0, -1 ) =>
    {
      if point.x == 0
      {
        &empty
      }
      else
      {
        &map[ point.y as usize ][ ( point.x as usize - 1 ) % w ]
      }
    },
    ( 1, -1 ) =>
    {
      if point.y == 0
      {
        &empty
      }
      else
      {
        &map[ ( point.y as usize - 1 ) % h ][ point.x as usize ]
      }
    },
    ( 0, 1 ) => &map[ point.y as usize ][ ( point.x as usize + 1 ) % w ],
    ( 1, 1 ) => &map[ ( point.y as usize + 1 ) % h ][ point.x as usize ],
    _ => unreachable!()
  }
  .clone()
}

/// Calculates current tile variants relatively to neighbour tiles
/// and only for [ `Relation::Isotropic` ]. [ `Relation::Anisotropic` ]
/// isn't yet implemented
fn calculate_isotropic_variants
(
  map : &Vec< Vec< Vec< u8 > > >,
  directions : &Vec< Direction >,
  relations : &Relations,
  point : Point
) -> Vec< u8 >
{
  let isotropic = relations.get_all_isotropic();
  if isotropic.is_empty()
  {
      let actual_variants = map[ point.y as usize ][ point.x as usize ].clone();
      return actual_variants;
  }

  // Get ruled neighbour variants for every possible variant of current
  // point and intersect with current point variants

  let new_variants = map[ point.y as usize ][ point.x as usize ]
  .iter()
  .cloned()
  .collect::< HashSet::< _ > >();
  new_variants.into_iter()
  .filter
  (
    | i |
    {
      let Relation::Isotropic( ref limited_variants ) = relations.0[ *i as usize ]
      else
      {
        unreachable!();
      };
      !directions.iter()
      .any(
        | d |
        {
          let set = get_neighbour_variants( map, point, d.difference() )
          .into_iter()
          .collect::< HashSet< _ > >();
          if set.is_empty()
          {
            return false;
          }
          set.intersection( &limited_variants )
          .next()
          .is_none()
        }
      )
    }
  )
  .collect::< Vec< _ > >()
}

/// Returns neighbour list for input tiles coordinates (Vec<Point>)
fn get_neighbours( points : Vec< Point >, diffs : Vec< ( usize, isize ) > ) -> Vec< Point >
{
  points.into_par_iter()
  .map
  (
    | p |
    {
      let mut points = vec![];
      for ( dim, diff ) in &diffs
      {
        points.push
        (
          match dim
          {
            0 => Point::new( p.x, p.y + diff ),
            1 => Point::new( p.x + diff, p.y ),
            _ => unreachable!()
          }
        );
      }
      points
    }
  )
  .flatten()
  .collect::< HashSet< _ > >()
  .into_iter()
  .collect::< Vec< _ > >()
}

/// Return default tiles set for each side of current map
fn default_edges() -> HashMap< Direction, Vec< u8 > >
{
  let mut edges = HashMap::new();
  let directions =
  [
    Direction::N,
    Direction::S,
    Direction::W,
    Direction::E
  ];
  for d in directions
  {
    edges.insert( d, vec![] );
  }
  edges
}

/// Calculates many random values from range
fn choose_multiple< T >( range : Range< T >, count : usize ) -> Vec< T >
where
  T: Clone +
  std::marker::Send +
  std::cmp::PartialOrd +
  rand::distributions::uniform::SampleUniform +
  std::marker::Sync
{
  ( 0..count )
  .into_par_iter()
  .map_init(
    ||
    {
      SmallRng::from_rng( rand::thread_rng() ).unwrap(),
      | r, _ |
      {
        r.gen_range( range.clone() )
      }
    }
  )
  .collect::< Vec< T > >()
}

/// Creates random strating front for [`Wfc`].
///
/// Arguments:
/// * density - front size in percent from map area
/// * size - map width and height,
/// * sample_size - all tiles variants count
///
fn create_new_front( density : f32, size : ( usize, usize ), sample_size : u8 ) -> Result< ( Vec< Point >, Vec< u8 > ), String >
{
  if 0.0 > density && density < 1.0
  {
    return Err( "density outside [0;1] range".to_string() );
  }
  let random_count = ( ( size.0 * size.1 ) as f32 * density ).floor() as usize;
  let x = choose_multiple::< isize >( 0..( size.0 as isize ), random_count );
  let y = choose_multiple::< isize >( 0..( size.1 as isize ), random_count );
  let v = choose_multiple::< u8 >( 0..sample_size, random_count );
  let front = x.into_iter()
  .zip( y.into_iter() )
  .map( | ( x, y ) | Point::new( x, y ) ).collect::< Vec< _ > >();
  Ok( ( front, v ) )
}

/// Generate map with one function call
pub fn generate
(
  size : ( usize, usize ),
  relations : Relations,
  density : f32
) -> Result< Vec< Vec< u8 > >, String >
{
  let ( front, v ) = create_new_front( density, size, relations.0.len() as u8 )?;
  let points_to_set = front.iter()
  .cloned()
  .zip( v.into_iter() )
  .collect::< Vec< ( Point, u8 ) > >();
  Wfc::new()
  .size( size )
  .edges( default_edges() )
  .set_points( points_to_set )
  .front( front )
  .relations( relations )
  .calculate()
}

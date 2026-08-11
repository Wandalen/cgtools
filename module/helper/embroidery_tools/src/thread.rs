//!
//! # Embroidery thread representation
//!

mod private
{
  use std::borrow::Cow;
  #[ cfg( feature = "random" ) ]
  use rand::seq::IndexedRandom;
  use itertools::Itertools as _;
  use crate::format::pec;

  /// RGB color
  // Fixed-shape 3-component RGB triple with no invariant between fields — direct
  // struct-literal construction is the deliberate public contract, pinned by
  // `tests/pes_test.rs` and documented in `readme.md`, so `#[non_exhaustive]`
  // would break that contract (same precedent as `browser_log::panic::Config`).
  #[ derive( Debug, Default, Clone, Copy, PartialEq, Eq, Hash ) ]
  pub struct Color
  {
    /// Red component
    pub r : u8,
    /// Green component
    pub g : u8,
    /// Blue component
    pub b : u8,
  }

  /// General Thread structure for storing information about threads
  /// used in embroidery file. Not all fields may be used. Depends on a format
  // Plain data record with no invariant between fields — direct struct-literal
  // construction (including `..Default::default()`) is the deliberate public
  // contract, pinned by `tests/pes_test.rs` and documented in `readme.md`, so
  // `#[non_exhaustive]` would break that contract (same precedent as `Color` above).
  #[ derive( Debug, Default, Clone, PartialEq, Eq, Hash ) ]
  pub struct Thread
  {
    /// Color of thread
    pub color : Color,
    /// Thread description, almost always it is shade name
    pub description : Cow< 'static, str >,
    /// A number in thread catalog
    pub catalog_number : Cow< 'static, str >,
    /// Some additional description
    pub details : Cow< 'static, str >,
    /// Brand name
    pub brand : Cow< 'static, str >,
    /// Chart name
    pub chart : Cow< 'static, str >,
    /// Weight of thread
    pub weight : Cow< 'static, str >,
  }

  /// Takes unique colors from `threadlist` and maps them by finding closest colors from `palette` for each unique color.
  /// # Returns
  /// Indices into `palette` for every color in `threadlist`
  /// # Panics
  /// Panics if `palette` is empty, since `chart` is then empty too and every
  /// lookup in `threadlist` has no candidate index to resolve to.
  #[ inline ]
  pub fn build_unique_palette( palette : &[ Thread ], threadlist : &[ Thread ] ) -> Vec< usize >
  {
    let mut chart = vec![ None; palette.len() ];
    let mut palette : Vec< _ > = palette.iter().map( Some ).collect();

    for thread in threadlist.iter().unique()
    {
      let index = find_nearest_color( &thread.color, &palette );
      if let Some( index ) = index
      {
        palette[ index ] = None;
        chart[ index ] = Some( thread );
      }
      else
      {
        break;
      }
    }

    let mut palette = vec![];
    for thread in threadlist
    {
      palette.push( find_nearest_color( &thread.color, &chart ).unwrap() );
    }

    palette
  }

  /// Finds index of closest color in palette.
  /// # Returns
  /// `None` if palette consists only of `None` values,
  /// otherwise returns index of closest color
  #[ must_use ]
  #[ inline ]
  pub fn find_nearest_color( color : &Color, palette : &[ Option< &Thread > ] ) -> Option< usize >
  {
    let mut closest_index = None;
    let mut current_distance = i32::MAX;

    for ( i, thread ) in palette.iter().enumerate()
    {
      if let Some( thread ) = thread
      {
        let dist = color_distance_red_mean( color, &thread.color );
        if dist <= current_distance
        {
          current_distance = dist;
          closest_index = Some( i );
        }
      }
    }

    closest_index
  }

  /// Calculates distance between colors
  #[ must_use ]
  #[ inline ]
  pub fn color_distance_red_mean( color1 : &Color, color2 : &Color ) -> i32
  {
    // See the very good color distance paper:
    // https://www.compuphase.com/cmetric.htm

    let red_mean = ( i32::from( color1.r ) + i32::from( color2.r ) ) / 2;
    let r = i32::from( color1.r ) - i32::from( color2.r );
    let g = i32::from( color1.g ) - i32::from( color2.g );
    let b = i32::from( color1.b ) - i32::from( color2.b );

    ( ( ( 512 + red_mean ) * r * r ) >> 8 )
    + 4 * g * g
    + ( ( ( 767 - red_mean ) * b * b ) >> 8 )
  }

  /// Retrieves a random thread from PEC pallete
  /// # Panics
  /// Never panics in practice: the PEC thread palette is a fixed 65-entry
  /// array, so the `1..` slice is always non-empty and `choose` always
  /// yields `Some`.
  #[ must_use ]
  #[ inline ]
  pub fn get_random_thread() -> Thread
  {
    #[ cfg( feature = "random" ) ]
    {
      pec::pec_threads()[ 1.. ].choose( &mut rand::rng() ).unwrap().clone()
    }
    #[ cfg( not( feature = "random" ) ) ]
    {
      pec::pec_threads()[ 1 ].clone()
    }
  }
}

crate::mod_interface!
{
  own use Thread;
  own use Color;
  own use build_unique_palette;
  own use find_nearest_color;
  own use color_distance_red_mean;
  own use get_random_thread;
}

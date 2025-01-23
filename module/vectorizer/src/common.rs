//! This module provides utilities for image processing, focusing on color analysis
//! and transparency detection within images. It includes functions to:
//!
//! - Find an unused color in an image for tasks like color-based masking.
//! - Check if a specific color is present within an image.
//! - Determine whether an image should be keyed, which is useful for processes like
//!   adding a green screen effect for later removal.
//!
//! # Constants
//! - `NUM_UNUSED_COLOR_ITERATIONS`: Number of attempts to generate random unused colors.
//! - `KEYING_THRESHOLD`: Proportion of transparent pixels required to trigger keying.
//!
//! # Functions
//! - `find_unused_color_in_image`: Identifies a color not present in the provided image.
//! - `color_exists_in_image`: Checks for the presence of a specific color in an image.
//! - `should_key_image`: Determines if an image requires keying based on its transparency.
//!
//! These tools are particularly useful in workflows involving image processing, masking,
//! or creating visual effects.

mod private
{
  use crate::*;
  use std::collections::HashMap;
  use fastrand::Rng;
  use palette::{ color_difference::EuclideanDistance, IntoColor, Lab, Srgb };
  use visioncortex::Color;

  const NUM_UNUSED_COLOR_ITERATIONS: usize = 6;
  const KEYING_THRESHOLD: f32 = 0.2;

  /// Find a color that is not present in a image.
  /// First tries to check a list of `special` colors, and the a list of randomly generated colors
  pub fn find_unused_color_in_image( img : &visioncortex::ColorImage ) -> Result < visioncortex::Color > 
  {
    let special_colors = IntoIterator::into_iter
    ([
        visioncortex::Color::new( 255, 0, 0 ),
        visioncortex::Color::new( 0, 255, 0 ),
        visioncortex::Color::new( 0, 0, 255 ),
        visioncortex::Color::new( 255, 255, 0 ),
        visioncortex::Color::new( 0, 255, 255 ),
        visioncortex::Color::new( 255, 0, 255 ),
    ]);
    let rng = Rng::new();
    let random_colors = ( 0..NUM_UNUSED_COLOR_ITERATIONS )
    .map( | _ | visioncortex::Color::new( rng.u8( ..) , rng.u8( .. ), rng.u8( .. ) ) );

    for color in special_colors.chain( random_colors ) 
    {
      if !color_exists_in_image( img, color ) 
      {
        return Ok( color );
      }
    }
    Err
    ( 
      Error::KeyColorError
    )
  }

  /// Checks if `color` is present in `img`
  pub fn color_exists_in_image( img : &visioncortex::ColorImage, color : visioncortex::Color ) -> bool 
  {
    for y in 0..img.height 
    {
      for x in 0..img.width 
      {
        let pixel_color = img.get_pixel( x, y );
        if pixel_color.r == color.r && 
           pixel_color.g == color.g && 
           pixel_color.b == color.b 
        {
          return true;
        }
      }
    }
    false
  }

  /// Checks if the image should be keyed (like adding a green screen, to cut it out later).
  ///
  /// This function determines whether an image requires keying based on its transparency. It scans several
  /// horizontal lines across the image to count the number of transparent pixels. If the proportion of transparent
  /// pixels exceeds a predefined threshold, the function returns `true`, indicating that the image should be keyed.
  ///
  /// # Arguments
  /// * `img` - A reference to a `visioncortex::ColorImage` representing the input image.
  ///
  /// # Returns
  /// * `bool` - `true` if the image should be keyed, `false` otherwise.
  pub fn should_key_image( img : &visioncortex::ColorImage ) -> bool 
  {
    let ( width, height ) = ( img.width, img.height );
    if width == 0 || height == 0 
    {
      return false;
    }

    // Check for transparency at several scanlines in y direction to know if the image needs to be keyed.
    // Should be keyed if the total amount of transparent pixels is bigger than the threshold
    let threshold = ( ( width * 2 ) as f32 * KEYING_THRESHOLD ) as usize;
    let mut num_transparent_pixels = 0;
    let y_positions = 
    [
      0,
      height / 4,
      height / 2,
      3 * height / 4,
      height - 1,
    ];

    for y in y_positions 
    {
      for x in 0..width 
      {
        if img.get_pixel( x, y ).a == 0 
        {
          num_transparent_pixels += 1;
        }
        if num_transparent_pixels >= threshold 
        {
          return true;
        }
      }
    }

    false
  }
  

  /// Return the background color of the image
  pub fn background_color( img : &visioncortex::ColorImage, mask : u8 ) -> Option< [ u8; 3 ] > 
  {
    let mut unique_colors = HashMap::new();

    for y in [ 0, img.height - 1 ]
    {
      for x in 0..img.width
      {
        let c = img.get_pixel( x, y );
        if c.a > 0
        {
          unique_colors.entry( [ c.r & mask, c.g & mask, c.b & mask ] )
          .and_modify( | v | *v += 1 )
          .or_insert( 1 );
        }
      }
    }

    for x in [ 0, img.width - 1 ]
    {
      for y in 0..img.height
      {
        let c = img.get_pixel( x, y );
        if c.a > 0
        {
          unique_colors.entry( [ c.r & mask, c.g & mask, c.b & mask ] )
          .and_modify( | v | *v += 1 )
          .or_insert( 1 );
        }
      }
    }

    let mut colors : Vec< ( [ u8; 3 ], u32 ) > = unique_colors.into_iter().collect();
    colors.sort_unstable_by_key( | ( _, count ) | *count );


    colors.last().map( | ( col, _ ) | *col )
  }

  /// Return the Euclid distance between two colors in CIELAB color space
  pub fn euclid_difference( c1 : Color, c2 : Color ) -> f32
  {
    let c1 =  Srgb::from( [ c1.r, c1.g, c1.b ] ).into_linear::< f32 >();
    let c2 =  Srgb::from( [ c2.r, c2.g, c2.b ] ).into_linear::< f32 >();
    let lab_c1 : Lab = c1.into_color();
    let lab_c2 : Lab = c2.into_color();
    lab_c1.distance_squared( lab_c2 )
  }
}


crate::mod_interface!
{

  orphan use
  {
    find_unused_color_in_image,
    color_exists_in_image,
    should_key_image,
    background_color,
    euclid_difference
  };
}

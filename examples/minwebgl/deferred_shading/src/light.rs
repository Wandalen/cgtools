//! Light system initialization and management

use minwebgl as gl;
use gl::{ GL, F32x3 };
use crate::{ types::LightSystem, elliptical_orbit::EllipticalOrbit };
use std::{ cell::RefCell, f32, rc::Rc };

/// Initialize the light system with random orbits and positions
pub fn create_light_system
(
  gl : &GL,
  max_count : usize,
  min_radius : f32,
  max_radius : f32
) -> Result< LightSystem, gl::WebglError >
{
  // Generate random elliptical orbits
  let orbits = ( 0..max_count ).map
  (
    | _ |
    EllipticalOrbit
    {
      center : F32x3::new
      (
        rand::random_range( -30.0..=30.0 ),
        rand::random_range( -30.0..=30.0 ),
        rand::random_range( -110.0..=-90.0 )
      ),
      ..EllipticalOrbit::random()
    }
  ).collect::< Vec< _ > >();

  let offsets = ( 0..max_count )
    .map( | _ | rand::random_range( 0.0..=( f32::consts::PI * 2.0 ) ) )
    .collect::< Vec< _ > >();

  // Generate radii
  let mut radii : Vec< f32 > = ( 0..max_count )
    .map( | _ | rand::random_range( min_radius..=max_radius ) )
    .collect();
  radii[ 0 ] = 100.0; // First light is global

  let radii = Rc::new( RefCell::new( radii ) );

  // Initialize translations
  let mut translations = vec![ [ 0.0f32, 0.0, 0.0 ]; max_count ];
  translations[ 0 ] = [ 0.0, 0.0, -100.0 ];

  // Create buffers
  let translation_buffer = gl::buffer::create( gl )?;
  gl::buffer::upload( gl, &translation_buffer, &translations, GL::DYNAMIC_DRAW );

  let radius_buffer = gl::buffer::create( gl )?;
  gl::buffer::upload( gl, &radius_buffer, radii.borrow().as_slice(), GL::DYNAMIC_DRAW );

  let prev_radius_range = Rc::new( RefCell::new( ( min_radius, max_radius ) ) );

  Ok
  (
    LightSystem
    {
      translations,
      translation_buffer,
      radii,
      radius_buffer,
      orbits,
      offsets,
      prev_radius_range,
      max_count,
    }
  )
}

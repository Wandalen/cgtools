//! Trajectory ribbons + sensor rings, ported from
//! `examples/threejs/falling_frontier/src/world/trajectories.js`, using
//! `line_tools::d3::Line` - the first user of `line_tools` in this crate
//! (see `PORT_PLAN.md`'s M1 note: deliberately *not* used for the base
//! grid, but exactly right for "a handful of rings/paths" like these).
//!
//! Deliberate simplification vs. the JS reference: waypoint ring markers
//! and the dashed height-guide-lines dropping from each waypoint to the
//! grid plane are dropped - decorative flourishes on top of a ribbon that's
//! already there, not load-bearing for reading the patrol route. Hidden by
//! default (`main.js`'s own `groups.trajectory.visible = false;
//! groups.sensorRing.visible = false;`), toggled via the dev panel.

use minwebgl as gl;
use gl::GL;
use line_tools::d3::{ Line, DashPattern };

use crate::ships::{ Ships, SHIP_COUNT, SHIP_Y };
use primitive_generation::spline;

const RIBBON_SAMPLES : usize = 80;
const RIBBON_WIDTH_PX : f32 = 2.0;

const SENSOR_RING_SEGMENTS : usize = 64;
const SENSOR_RING_WIDTH_PX : f32 = 1.5;
// COLORS.sensorRing (0x00a3cc) from
// examples/threejs/falling_frontier/src/config/colors.js.
const SENSOR_RING_COLOR : [ f32; 3 ] = [ 0.0, 0.6392, 0.8 ];
// Slightly above the grid plane, matches trajectories.js's own comment.
const SENSOR_RING_Y : f32 = 0.2;

pub struct Trajectories
{
  ribbons : Vec< Line >,
  sensor_rings : Vec< Line >,
}

impl Trajectories
{
  /// Builds one ribbon `Line` per ship (sampled from the same Catmull-Rom
  /// spline `Ships::advance` drives motion with) and one dashed sensor-ring
  /// `Line` per ship that defines a `sensor_radius`, centered on that
  /// ship's *starting* position - matches the JS reference, which builds
  /// `createSensorRing(ship.position, ...)` once at startup rather than
  /// tracking the ship live.
  pub fn new( gl : &GL, ships : &Ships, projection : gl::F32x4x4, resolution : [ f32; 2 ] ) -> Result< Self, gl::WebglError >
  {
    let mut ribbons = Vec::with_capacity( SHIP_COUNT );
    for i in 0 .. SHIP_COUNT
    {
      let path = ships.path( i );
      let mut line = Line::default();
      line.vertex_color_use( false );
      line.world_units_use( false );
      line.mesh_create( gl, None )?;

      for s in 0 ..= RIBBON_SAMPLES
      {
        let t = s as f32 / RIBBON_SAMPLES as f32;
        let p = spline::point_at_progress( path, t );
        line.point_add_back( &[ p[ 0 ], SHIP_Y, p[ 1 ] ] );
      }

      let mesh = line.mesh_get_mut()?;
      mesh.upload( gl, "u_width", &RIBBON_WIDTH_PX )?;
      mesh.upload( gl, "u_color", &gl::F32x3::from( ships.trajectory_color( i ) ) )?;
      mesh.upload( gl, "u_resolution", &gl::F32x2::from( resolution ) )?;
      mesh.upload( gl, "u_projection_matrix", &projection )?;
      mesh.upload( gl, "u_world_matrix", &gl::F32x4x4::identity() )?;
      ribbons.push( line );
    }

    let mut sensor_rings = Vec::new();
    for i in 0 .. SHIP_COUNT
    {
      let Some( radius ) = ships.sensor_radius( i ) else { continue };
      let center = ships.position( i );

      let mut line = Line::default();
      line.vertex_color_use( false );
      line.world_units_use( false );
      line.dash_use( true );
      // dashSize: 3, gapSize: 2 from trajectories.js's createSensorRing.
      line.dash_pattern_set( DashPattern::V2( [ 3.0, 2.0 ] ) );
      line.mesh_create( gl, None )?;

      for s in 0 ..= SENSOR_RING_SEGMENTS
      {
        let theta = ( s as f32 / SENSOR_RING_SEGMENTS as f32 ) * std::f32::consts::TAU;
        line.point_add_back( &[ center[ 0 ] + theta.cos() * radius, SENSOR_RING_Y, center[ 1 ] + theta.sin() * radius ] );
      }

      let mesh = line.mesh_get_mut()?;
      mesh.upload( gl, "u_width", &SENSOR_RING_WIDTH_PX )?;
      mesh.upload( gl, "u_color", &gl::F32x3::from( SENSOR_RING_COLOR ) )?;
      mesh.upload( gl, "u_resolution", &gl::F32x2::from( resolution ) )?;
      mesh.upload( gl, "u_projection_matrix", &projection )?;
      mesh.upload( gl, "u_world_matrix", &gl::F32x4x4::identity() )?;
      mesh.upload( gl, "u_dash_offset", &0.0f32 )?;
      sensor_rings.push( line );
    }

    Ok( Self { ribbons, sensor_rings } )
  }

  /// Uploads the frame's view/projection/resolution to every visible line
  /// and draws it. `show_ribbons`/`show_sensor_rings` gate each group
  /// independently, matching the dev panel's own two checkboxes (and the
  /// JS reference's separate `toggle-trajectories`/`toggle-ranges`
  /// buttons).
  pub fn draw
  (
    &mut self, gl : &GL, view : gl::F32x4x4, projection : gl::F32x4x4, resolution : [ f32; 2 ],
    show_ribbons : bool, show_sensor_rings : bool,
  )
  {
    let groups : [ ( &mut Vec< Line >, bool ); 2 ] =
    [
      ( &mut self.ribbons, show_ribbons ),
      ( &mut self.sensor_rings, show_sensor_rings ),
    ];
    for ( lines, visible ) in groups
    {
      if !visible { continue; }
      for line in lines
      {
        let mesh = line.mesh_get_mut().unwrap();
        mesh.upload( gl, "u_view_matrix", &view ).unwrap();
        mesh.upload( gl, "u_projection_matrix", &projection ).unwrap();
        mesh.upload( gl, "u_resolution", &gl::F32x2::from( resolution ) ).unwrap();
        line.draw( gl ).unwrap();
      }
    }
  }
}

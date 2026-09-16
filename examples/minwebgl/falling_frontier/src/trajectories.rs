//! Trajectory ribbons, ported from
//! `examples/threejs/falling_frontier/src/world/trajectories.js`, using
//! `line_tools::d3::Line` - the first user of `line_tools` in this crate
//! (see `PORT_PLAN.md`'s M1 note: deliberately *not* used for the base
//! grid, but exactly right for "a handful of rings/paths" like these).
//!
//! Deliberate simplification vs. the JS reference: waypoint ring markers
//! and the dashed height-guide-lines dropping from each waypoint to the
//! grid plane are dropped - decorative flourishes on top of a ribbon that's
//! already there, not load-bearing for reading the patrol route. Hidden by
//! default (`main.js`'s own `groups.trajectory.visible = false;`), toggled
//! via the dev panel.
//!
//! The JS reference's sensor rings (`createSensorRing`, a dashed radius
//! circle per ship with a `sensorRadius`) are cut entirely, not just hidden -
//! they never carried gameplay meaning in this port, only decoration.

use minwebgl as gl;
use gl::GL;
use line_tools::d3::Line;

use crate::ships::{ Ships, SHIP_COUNT, SHIP_Y };
use primitive_generation::spline;

const RIBBON_SAMPLES : usize = 80;
const RIBBON_WIDTH_PX : f32 = 2.0;

pub struct Trajectories
{
  ribbons : Vec< Line >,
}

impl Trajectories
{
  /// Builds one ribbon `Line` per ship, sampled from the same Catmull-Rom
  /// spline `Ships::advance` drives motion with.
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

    Ok( Self { ribbons } )
  }

  /// Uploads the frame's view/projection/resolution to every ribbon and
  /// draws it, if `show_ribbons` (the dev panel's "Trajectories" toggle) is
  /// set.
  pub fn draw( &mut self, gl : &GL, view : gl::F32x4x4, projection : gl::F32x4x4, resolution : [ f32; 2 ], show_ribbons : bool )
  {
    if !show_ribbons { return; }
    for line in &mut self.ribbons
    {
      let mesh = line.mesh_get_mut().unwrap();
      mesh.upload( gl, "u_view_matrix", &view ).unwrap();
      mesh.upload( gl, "u_projection_matrix", &projection ).unwrap();
      mesh.upload( gl, "u_resolution", &gl::F32x2::from( resolution ) ).unwrap();
      line.draw( gl ).unwrap();
    }
  }
}

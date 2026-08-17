//! M6 transform gizmo: two simple handle meshes (a flat XZ "move" cross and
//! a flat "rotate" ring), drawn at the selected object's position with
//! their own tiny unlit program - drawn with depth test off so the handle
//! stays visible/clickable regardless of what's in front of it. No gizmo
//! existed anywhere in cgtools before this.
//!
//! Deliberate simplification vs. the JS reference: three.js's
//! `TransformControls` draws a separate arrow per axis (X/Z) plus a plane
//! handle, all individually pickable; this draws one handle per mode
//! instead (translate is XZ-only in this scene anyway - see
//! `examples/threejs/falling_frontier/src/interaction/transform.js`'s
//! `MODE_CONSTRAINTS`, `showY: false` - so a single free-drag-in-plane
//! handle covers the same freedom without per-axis hit-testing).
//!
//! The handle is picked through the same GPU id-buffer mechanism as scene
//! objects (`picking.rs`) - `main.rs` reserves one more id (`GIZMO_ID`) and
//! feeds `Gizmo::part`'s `HullPart` into the same id pass alongside
//! asteroids/ships/station whenever something is selected.

use minwebgl as gl;
use gl::GL;

use crate::hull::{ upload_mesh, HullPart };
use crate::primitives::{ box_mesh, torus_mesh };

pub const TRANSLATE_COLOR : [ f32; 3 ] = [ 1.0, 0.85, 0.1 ];
pub const ROTATE_COLOR : [ f32; 3 ] = [ 1.0, 0.25, 0.85 ];
const HANDLE_ALPHA : f32 = 0.85;

#[ derive( Clone, Copy, PartialEq, Eq ) ]
pub enum GizmoMode
{
  Translate,
  Rotate,
}

struct GizmoUniforms
{
  view_proj : Option< gl::WebGlUniformLocation >,
  model : Option< gl::WebGlUniformLocation >,
  color : Option< gl::WebGlUniformLocation >,
  alpha : Option< gl::WebGlUniformLocation >,
}

pub struct Gizmo
{
  translate_vao : gl::WebGlVertexArrayObject,
  translate_index_count : i32,
  rotate_vao : gl::WebGlVertexArrayObject,
  rotate_index_count : i32,
  program : gl::WebGlProgram,
  uniforms : GizmoUniforms,
}

impl Gizmo
{
  pub fn new( gl : &GL ) -> Self
  {
    // Translate handle: two thin boxes crossing at the origin, along local
    // X and Z - reads as a flat "+" you can grab anywhere on to drag the
    // object freely within the XZ plane.
    let ( mut positions, mut indices ) = box_mesh( 6.0, 0.25, 0.7 );
    let ( x_positions, x_indices ) = box_mesh( 0.7, 0.25, 6.0 );
    let offset = positions.len() as u32;
    positions.extend( x_positions );
    indices.extend( x_indices.into_iter().map( | i | i + offset ) );
    let ( translate_vao, translate_index_count ) = upload_mesh( gl, &positions, &indices );

    // Rotate handle: a ring flat in the XZ plane (torus_mesh's own ring
    // lies in XY by default - rotate 90° around X on the CPU, same
    // reorientation `station.rs` applies to its ring via a model-matrix
    // rotation; done here on the vertices instead since the gizmo's model
    // matrix is the bare object transform with no room for an extra local
    // offset).
    let ( rotate_positions, rotate_indices ) = torus_mesh( 10.0, 0.35, 8, 24 );
    let rotate_positions : Vec< [ f32; 3 ] > = rotate_positions.iter()
    .map( | v | [ v[ 0 ], -v[ 2 ], v[ 1 ] ] )
    .collect();
    let ( rotate_vao, rotate_index_count ) = upload_mesh( gl, &rotate_positions, &rotate_indices );

    let vertex_shader = include_str!( "shaders/gizmo.vert" );
    let fragment_shader = include_str!( "shaders/gizmo.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( gl )
    .unwrap();

    let uniforms = GizmoUniforms
    {
      view_proj : gl.get_uniform_location( &program, "u_view_proj" ),
      model : gl.get_uniform_location( &program, "u_model" ),
      color : gl.get_uniform_location( &program, "u_color" ),
      alpha : gl.get_uniform_location( &program, "u_alpha" ),
    };

    Self { translate_vao, translate_index_count, rotate_vao, rotate_index_count, program, uniforms }
  }

  /// Builds the current handle as a `HullPart` at `object_transform` -
  /// shared between the visible draw (`Gizmo::draw`) and the id pass
  /// (`picking.rs`'s `PickBuffer::render`, which only needs `.model`/
  /// `.pick_id`/`.vao`/`.index_count` from any `HullPart`).
  pub fn part( &self, mode : GizmoMode, object_transform : gl::F32x4x4, pick_id : i32 ) -> HullPart
  {
    let ( vao, index_count, color ) = match mode
    {
      GizmoMode::Translate => ( self.translate_vao.clone(), self.translate_index_count, TRANSLATE_COLOR ),
      GizmoMode::Rotate => ( self.rotate_vao.clone(), self.rotate_index_count, ROTATE_COLOR ),
    };
    HullPart
    {
      vao, index_count,
      local_transform : gl::F32x4x4::identity(),
      model : object_transform,
      color, ambient : 1.0,
      pick_id,
    }
  }

  /// Draws `part` (as built by `Gizmo::part`) with depth test off, so the
  /// handle stays visible/clickable through the object it belongs to.
  /// Restores depth test afterward - everything else in this crate assumes
  /// it's always on.
  pub fn draw( &self, gl : &GL, view_proj : gl::F32x4x4, part : &HullPart )
  {
    gl.use_program( Some( &self.program ) );
    let u = &self.uniforms;
    gl::uniform::matrix_upload( gl, u.view_proj.clone(), view_proj.to_array().as_slice(), true ).unwrap();
    gl::uniform::matrix_upload( gl, u.model.clone(), part.model.to_array().as_slice(), true ).unwrap();
    gl::uniform::upload( gl, u.color.clone(), part.color.as_slice() ).unwrap();
    gl::uniform::upload( gl, u.alpha.clone(), &HANDLE_ALPHA ).unwrap();

    gl.disable( GL::DEPTH_TEST );
    gl.enable( GL::BLEND );
    gl.blend_func( GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA );

    gl.bind_vertex_array( Some( &part.vao ) );
    gl.draw_elements_with_i32( GL::TRIANGLES, part.index_count, GL::UNSIGNED_INT, 0 );

    gl.disable( GL::BLEND );
    gl.enable( GL::DEPTH_TEST );
  }
}

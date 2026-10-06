//! WebGL adapter sprite and mesh renderers.
//!
//! Extracted from `webgl.rs` to keep that file under the per-source-file size
//! budget. Each renderer owns its single-draw and instanced-batch shader
//! programs and issues the draw calls; `WebGlBackend` resolves resources,
//! blend state and the premultiplied flag before calling in.

mod private
{
  use minwebgl as gl;
  use super::super::webgl_helpers::{ GpuResources, GpuGeometry, GpuBatch, topology_to_gl };
  use crate::types::FillRef;

  /// The tint every fragment shader applies ( `shaders/tint.glsl` ), kept in one
  /// place because its premultiplied branch is what the `ONE` source factor of
  /// `blend_apply` relies on.
  const TINT_CHUNK : &str = include_str!( "../shaders/tint.glsl" );

  /// Splices [`TINT_CHUNK`] into a fragment shader at its `#include "tint.glsl"`
  /// line. GLSL ES has no `#include`, so a shader whose line went missing fails
  /// to compile instead of silently drawing untinted.
  fn fragment_source( source : &str ) -> String
  {
    source.replacen( "#include \"tint.glsl\"", TINT_CHUNK, 1 )
  }

  // ============================================================================
  // Sprite renderer
  // ============================================================================

  /// Handles single sprite draws and sprite batch instancing.
  /// Quad is generated in vertex shader from `gl_VertexID` (triangle strip, 4 vertices).
  pub struct SpriteRenderer
  {
    program : gl::Program,
    batch_program : gl::Program,
  }

  impl SpriteRenderer
  {
    /// Compiles the single-draw and instanced-batch programs.
    ///
    /// # Errors
    /// Returns the compile or link error of either program.
    pub fn new( gl : &gl::GL ) -> Result< Self, gl::WebglError >
    {
      let program = gl::Program::new
      (
        gl.clone(),
        include_str!( "../shaders/sprite.vert" ),
        &fragment_source( include_str!( "../shaders/sprite.frag" ) ),
      )?;
      let batch_program = gl::Program::new
      (
        gl.clone(),
        include_str!( "../shaders/sprite_batch.vert" ),
        &fragment_source( include_str!( "../shaders/sprite_batch.frag" ) ),
      )?;
      Ok( Self { program, batch_program } )
    }

    /// Draw a single sprite as a textured quad (triangle strip, 4 vertices from `gl_VertexID`).
    ///
    /// `region` is the sprite rect in pixels and `tex_size` is the sheet's dimensions — same
    /// convention as `sprite_batch.vert`, so both shaders normalize UV the same way.
    #[ allow( clippy::too_many_arguments, reason = "each parameter is a distinct WebGL uniform upload target; grouping into a struct would add indirection without reducing call-site complexity for this single-call-site private method" ) ]
    pub fn draw( &self, gl : &gl::GL, transform : &[ f32; 9 ], region : &[ f32; 4 ], tex_size : [ f32; 2 ], tint : &[ f32; 4 ], premultiplied : bool, viewport : [ f32; 2 ], depth : f32, max_depth : f32 )
    {
      // Unbind any VAO to prevent stale attribute state from interfering
      gl.bind_vertex_array( None );
      self.program.activate();
      self.program.uniform_matrix_upload( "u_transform", transform.as_slice(), true );
      self.program.uniform_upload( "u_region", region );
      self.program.uniform_upload( "u_tex_size", &tex_size );
      self.program.uniform_upload( "u_tint", tint );
      self.program.uniform_upload( "u_premultiplied", &i32::from( premultiplied ) );
      self.program.uniform_upload( "u_viewport", &viewport );
      self.program.uniform_upload( "u_depth", &depth );
      self.program.uniform_upload( "u_max_depth", &max_depth );
      gl.draw_arrays( gl::TRIANGLE_STRIP, 0, 4 );
    }

    /// Draw an instanced sprite batch. `premultiplied` is the flag `cmd_draw_batch`
    /// already handed to `blend_apply`, so the shader and the blend agree.
    pub fn batch_draw( &self, gl : &gl::GL, batch : &GpuBatch, resources : &GpuResources, premultiplied : bool, viewport : [ f32; 2 ], max_depth : f32 )
    {
      let GpuBatch::Sprite { instances, vao, params, .. } = batch else { return; };
      if instances.is_empty() { return; }

      let Some( gpu_tex ) = resources.texture( params.sheet ) else { return; };
      let tw = gpu_tex.width.get();
      let th = gpu_tex.height.get();
      if tw == 0 || th == 0 { return; }

      gl.active_texture( gl::TEXTURE0 );
      gl.bind_texture( gl::TEXTURE_2D, Some( &gpu_tex.texture ) );

      self.batch_program.activate();
      self.batch_program.uniform_upload( "u_viewport", &viewport );
      self.batch_program.uniform_upload( "u_tex_size", &[ tw as f32, th as f32 ] );
      self.batch_program.uniform_upload( "u_premultiplied", &i32::from( premultiplied ) );
      let parent_mat = params.transform.to_mat3();
      self.batch_program.uniform_matrix_upload( "u_parent", &parent_mat, true );
      self.batch_program.uniform_upload( "u_parent_depth", &params.transform.depth );
      self.batch_program.uniform_upload( "u_max_depth", &max_depth );

      gl.bind_vertex_array( Some( vao ) );
      gl.draw_arrays_instanced( gl::TRIANGLE_STRIP, 0, 4, instances.len() as i32 );
      // Unbind the batch VAO so subsequent GL state setup (e.g. a later
      // vertex_attrib_pointer call during batch construction) cannot
      // accidentally mutate this batch's attribute layout. The single-draw
      // path (`SpriteRenderer::draw`) likewise unbinds on exit, so both
      // sprite draw paths leave VAO 0 bound.
      gl.bind_vertex_array( None );
    }
  }

  // ============================================================================
  // Mesh renderer
  // ============================================================================

  /// Handles single mesh draws and mesh batch instancing.
  pub struct MeshRenderer
  {
    program : gl::Program,
    batch_program : gl::Program,
  }

  impl MeshRenderer
  {
    /// Compiles the single-draw and instanced-batch programs.
    ///
    /// # Errors
    /// Returns the compile or link error of either program.
    pub fn new( gl : &gl::GL ) -> Result< Self, gl::WebglError >
    {
      let program = gl::Program::new
      (
        gl.clone(),
        include_str!( "../shaders/mesh.vert" ),
        &fragment_source( include_str!( "../shaders/mesh.frag" ) ),
      )?;
      let batch_program = gl::Program::new
      (
        gl.clone(),
        include_str!( "../shaders/mesh_batch.vert" ),
        &fragment_source( include_str!( "../shaders/mesh_batch.frag" ) ),
      )?;
      Ok( Self { program, batch_program } )
    }

    /// Draw a single mesh.
    #[ allow( clippy::too_many_arguments, reason = "each parameter is a distinct WebGL uniform upload target or draw-call input; grouping into a struct would add indirection without reducing call-site complexity for this single-call-site private method" ) ]
    pub fn draw
    (
      &self,
      gl : &gl::GL,
      geom : &GpuGeometry,
      transform : &[ f32; 9 ],
      color : &[ f32; 4 ],
      topology : u32,
      viewport : [ f32; 2 ],
      use_texture : bool,
      premultiplied : bool,
      depth : f32,
      max_depth : f32,
    )
    {
      self.program.activate();
      self.program.uniform_matrix_upload( "u_transform", transform.as_slice(), true );
      self.program.uniform_upload( "u_color", color );
      self.program.uniform_upload( "u_viewport", &viewport );
      self.program.uniform_upload( "u_use_texture", &i32::from( use_texture ) );
      self.program.uniform_upload( "u_premultiplied", &i32::from( premultiplied ) );
      self.program.uniform_upload( "u_depth", &depth );
      self.program.uniform_upload( "u_max_depth", &max_depth );

      gl.bind_vertex_array( Some( &geom.vao ) );

      if let Some( ( count, gl_type ) ) = geom.index_count
      {
        gl.draw_elements_with_i32( topology, count as i32, gl_type, 0 );
      }
      else
      {
        gl.draw_arrays( topology, 0, geom.vertex_count as i32 );
      }
      // Unbind the geometry VAO so a subsequent `vertex_attrib_pointer` call
      // (e.g. during `mesh_batch_vao_setup` for another batch) cannot silently
      // mutate this geometry's attribute layout.
      gl.bind_vertex_array( None );
    }

    /// Draw an instanced mesh batch. VAO is already configured via `mesh_batch_vao_setup`.
    /// `premultiplied` is the flag `cmd_draw_batch` already handed to `blend_apply`,
    /// so the shader and the blend agree.
    pub fn batch_draw( &self, gl : &gl::GL, batch : &GpuBatch, resources : &GpuResources, premultiplied : bool, viewport : [ f32; 2 ], max_depth : f32 )
    {
      let GpuBatch::Mesh { instances, vao, params, .. } = batch else { return };
      if instances.is_empty() { return; }

      let Some( geom ) = resources.geometry( params.geometry ) else { return };
      let color = match params.fill { FillRef::Solid( c ) => c, _ => [ 1.0, 1.0, 1.0, 1.0 ] };
      let topology = topology_to_gl( &params.topology );

      let mut use_texture = false;
      if let Some( tex_id ) = params.texture
        && let Some( gpu_tex ) = resources.texture( tex_id )
      {
        // Fix(BUG-537): a pending image is skipped, as in `cmd_mesh`.
        if gpu_tex.width.get() == 0 || gpu_tex.height.get() == 0 { return; }
        gl.active_texture( gl::TEXTURE0 );
        gl.bind_texture( gl::TEXTURE_2D, Some( &gpu_tex.texture ) );
        use_texture = true;
      }

      self.batch_program.activate();
      self.batch_program.uniform_upload( "u_viewport", &viewport );
      self.batch_program.uniform_upload( "u_color", &color );
      self.batch_program.uniform_upload( "u_use_texture", &i32::from( use_texture ) );
      self.batch_program.uniform_upload( "u_premultiplied", &i32::from( premultiplied ) );
      let parent_mat = params.transform.to_mat3();
      self.batch_program.uniform_matrix_upload( "u_parent", &parent_mat, true );
      self.batch_program.uniform_upload( "u_parent_depth", &params.transform.depth );
      self.batch_program.uniform_upload( "u_max_depth", &max_depth );

      gl.bind_vertex_array( Some( vao ) );

      if let Some( ( count, gl_type ) ) = geom.index_count
      {
        gl.draw_elements_instanced_with_i32( topology, count as i32, gl_type, 0, instances.len() as i32 );
      }
      else
      {
        gl.draw_arrays_instanced( topology, 0, geom.vertex_count as i32, instances.len() as i32 );
      }
      // Unbind the batch VAO so subsequent GL state setup (e.g. a later
      // vertex_attrib_pointer call during batch construction) cannot
      // accidentally mutate this batch's attribute layout. The single-draw
      // path (`MeshRenderer::draw`) likewise unbinds on exit, so both mesh
      // draw paths leave VAO 0 bound.
      gl.bind_vertex_array( None );
    }
  }
}

mod_interface::mod_interface!
{
  own use SpriteRenderer;
  own use MeshRenderer;
}

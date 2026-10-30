//! WebGL adapter sprite and mesh renderers.
//!
//! Extracted from `webgl.rs` to keep that file under the per-source-file size
//! budget. Each renderer owns its single-draw and instanced-batch shader
//! programs and issues the draw calls; `WebGlBackend` resolves resources,
//! blend state and the premultiplied flag before calling in.

mod private
{
  use minwebgl as gl;
  use super::super::webgl_helpers::{ GpuResources, GpuGeometry, GpuBatch, mat3_mul, topology_to_gl };
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

  /// Prepend `#define <name>` to a GLSL source, inserted right AFTER the
  /// `#version` line (which GLSL requires to be first). Used to compile the two
  /// `sprite_batch.frag` variants (cutout / plain) from one source of truth.
  fn inject_define( src : &str, name : &str ) -> String
  {
    match src.split_once( '\n' )
    {
      Some( ( first, rest ) ) => format!( "{first}\n#define {name}\n{rest}" ),
      None => src.to_string(),
    }
  }

  // ============================================================================
  // Sprite renderer
  // ============================================================================

  /// Handles single sprite draws and sprite batch instancing.
  /// Quad is generated in vertex shader from `gl_VertexID` (triangle strip, 4 vertices).
  pub struct SpriteRenderer
  {
    program : gl::Program,
    /// Cutout variant of the batch shader (`#define USE_ALPHA_CLIP`): has the
    /// `discard`, used for `alpha_clip > 0` batches (cutout tiles).
    batch_program : gl::Program,
    /// Plain variant with NO `discard`, used for `alpha_clip == 0` batches.
    /// Discard-free so the GPU keeps early-Z and rejects their fragments under
    /// a nearer opaque layer before the fragment shader runs — the fillrate
    /// win for stacked full-screen layers.
    batch_program_noclip : gl::Program,
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
      let batch_vert = include_str!( "../shaders/sprite_batch.vert" );
      let batch_frag = fragment_source( include_str!( "../shaders/sprite_batch.frag" ) );
      let batch_program = gl::Program::new
      (
        gl.clone(),
        batch_vert,
        &inject_define( &batch_frag, "USE_ALPHA_CLIP" ),
      )?;
      let batch_program_noclip = gl::Program::new
      (
        gl.clone(),
        batch_vert,
        &batch_frag,
      )?;
      Ok( Self { program, batch_program, batch_program_noclip } )
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

    /// Draw an instanced sprite batch. `camera` is the world→screen view matrix
    /// (column-major 3×3), pre-multiplied into the batch parent so world-space
    /// instances are projected on the GPU (a pan/zoom updates this uniform only,
    /// leaving the per-instance buffers untouched). `premultiplied` is the flag
    /// `cmd_draw_batch` already handed to `blend_apply`, so the shader and the
    /// blend agree.
    #[ allow( clippy::too_many_arguments, reason = "each parameter is a distinct WebGL uniform upload target; grouping into a struct would add indirection without reducing call-site complexity for this single-call-site method" ) ]
    pub fn batch_draw( &self, gl : &gl::GL, batch : &GpuBatch, resources : &GpuResources, camera : &[ f32; 9 ], premultiplied : bool, viewport : [ f32; 2 ], max_depth : f32 )
    {
      let GpuBatch::Sprite { instances, vao, params, .. } = batch else { return; };
      // Draw only what the last flush put on the GPU (see `ArrayBuffer::gpu_len`).
      if instances.gpu_len() == 0 { return; }

      let Some( gpu_tex ) = resources.texture( params.sheet ) else { return; };
      let tw = gpu_tex.width.get();
      let th = gpu_tex.height.get();
      if tw == 0 || th == 0 { return; }

      gl.active_texture( gl::TEXTURE0 );
      gl.bind_texture( gl::TEXTURE_2D, Some( &gpu_tex.texture ) );

      // Pick the shader variant by whether this batch actually clips. Only
      // `alpha_clip > 0` batches need the `discard`; the rest use the
      // discard-free program so the GPU keeps early-Z (see the frag shader).
      let clips = params.alpha_clip > 0.0;
      let program = if clips { &self.batch_program } else { &self.batch_program_noclip };
      program.activate();
      program.uniform_upload( "u_viewport", &viewport );
      program.uniform_upload( "u_tex_size", &[ tw as f32, th as f32 ] );
      program.uniform_upload( "u_premultiplied", &i32::from( premultiplied ) );
      let parent_mat = mat3_mul( camera, &params.transform.to_mat3() );
      program.uniform_matrix_upload( "u_parent", &parent_mat, true );
      program.uniform_upload( "u_parent_depth", &params.transform.depth );
      program.uniform_upload( "u_max_depth", &max_depth );
      // Coverage cut-off (cutout variant only): >0 discards low-alpha texels
      // so transparent quad corners don't seal the depth buffer.
      if clips
      {
        program.uniform_upload( "u_alpha_clip", &params.alpha_clip );
      }

      gl.bind_vertex_array( Some( vao ) );
      gl.draw_arrays_instanced( gl::TRIANGLE_STRIP, 0, 4, instances.gpu_len() as i32 );
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
    /// `camera` is the world→screen view matrix, composed into the batch parent (see
    /// the sprite `batch_draw`). `premultiplied` is the flag `cmd_draw_batch` already
    /// handed to `blend_apply`, so the shader and the blend agree.
    #[ allow( clippy::too_many_arguments, reason = "each parameter is a distinct WebGL uniform upload target; grouping into a struct would add indirection without reducing call-site complexity for this single-call-site method" ) ]
    pub fn batch_draw( &self, gl : &gl::GL, batch : &GpuBatch, resources : &GpuResources, camera : &[ f32; 9 ], premultiplied : bool, viewport : [ f32; 2 ], max_depth : f32 )
    {
      let GpuBatch::Mesh { instances, vao, params, .. } = batch else { return };
      // Draw only what the last flush put on the GPU (see `ArrayBuffer::gpu_len`).
      if instances.gpu_len() == 0 { return; }

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
      let parent_mat = mat3_mul( camera, &params.transform.to_mat3() );
      self.batch_program.uniform_matrix_upload( "u_parent", &parent_mat, true );
      self.batch_program.uniform_upload( "u_parent_depth", &params.transform.depth );
      self.batch_program.uniform_upload( "u_max_depth", &max_depth );

      gl.bind_vertex_array( Some( vao ) );

      if let Some( ( count, gl_type ) ) = geom.index_count
      {
        gl.draw_elements_instanced_with_i32( topology, count as i32, gl_type, 0, instances.gpu_len() as i32 );
      }
      else
      {
        gl.draw_arrays_instanced( topology, 0, geom.vertex_count as i32, instances.gpu_len() as i32 );
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

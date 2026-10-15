mod private
{
  use naga::back::glsl;

  /// GLSL ES 300 translation of one WGSL module's vertex and fragment
  /// stages, produced by [`wgsl_to_webgl_glsl`] for gpu_hal's WebGL backend
  /// ( `Device::shader_module_create` ).
  #[ derive( Debug, Clone ) ]
  pub struct WebglGlslSource
  {
    /// Vertex stage GLSL ES 300 source, with every uniform block and
    /// texture/sampler uniform renamed per gpu_hal's WebGL introspection
    /// convention — see [`wgsl_to_webgl_glsl`].
    pub vertex : String,
    /// Fragment stage GLSL ES 300 source, with every uniform block and
    /// texture/sampler uniform renamed per gpu_hal's WebGL introspection
    /// convention — see [`wgsl_to_webgl_glsl`].
    pub fragment : String
  }

  /// Parses, validates, and translates `wgsl` into GLSL ES 300 for gpu_hal's
  /// WebGL backend, reading `vertex_entry`/`fragment_entry` as the WGSL
  /// entry point for each stage.
  ///
  /// Every uniform block naga generates a name for is renamed to
  /// `ub_{group}_{binding}`, and every combined texture/sampler uniform to
  /// `tex_{group}_{binding}` ( the texture's own binding — GLSL ES combines
  /// a WGSL `texture`/`sampler` pair into one `samplerN` uniform, and
  /// gpu_hal's introspection never looks up the paired sampler by name ),
  /// each read from that global variable's actual `@group`/`@binding` WGSL
  /// attributes — the exact convention `Device`'s WebGL binding
  /// introspection resolves at runtime ( `webgl_bindings_introspect`,
  /// `device.rs` ) — so callers never hand-guess a binding name, for any
  /// number of uniform blocks or textures, in either stage.
  ///
  /// Intended for use from a downstream crate's `build.rs` as a
  /// build-dependency ( `webgl-glsl-build` feature ) — this function and its
  /// `naga` dependency never become part of the compiled artifact, native or
  /// wasm32, and carry none of the `webgl` feature's wasm32-only
  /// `minwebgl` / `web-sys` dependencies.
  ///
  /// # Errors
  /// Returns a human-readable message on WGSL parse/validation failure, or
  /// if GLSL generation fails for either stage — meant to be interpolated
  /// directly into a `build.rs` `panic!`.
  pub fn wgsl_to_webgl_glsl
  (
    wgsl : &str,
    vertex_entry : &str,
    fragment_entry : &str,
  ) -> Result< WebglGlslSource, String >
  {
    let module = naga::front::wgsl::parse_str( wgsl )
    .map_err( | e | format!( "WGSL parse failed :: {e}" ) )?;
    let info = naga::valid::Validator::new( naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all() )
    .validate( &module )
    .map_err( | e | format!( "WGSL validation failed :: {e}" ) )?;

    // WebGL2 = GLSL ES 300. `writer_flags` deliberately omits
    // `ADJUST_COORDINATE_SPACE` ( naga's `Options::default()` sets it ) :
    // gpu_hal's WebGL backend expects plain GLSL with no NDC Y-flip, the
    // same convention its hand-written GLSL shaders already use.
    let options = glsl::Options
    {
      version : glsl::Version::Embedded { version : 300, is_webgl : true },
      writer_flags : glsl::WriterFlags::empty(),
      ..glsl::Options::default()
    };

    let ( vertex, vertex_reflection ) = stage_translate( &module, &info, &options, naga::ShaderStage::Vertex, vertex_entry )?;
    let ( fragment, fragment_reflection ) = stage_translate( &module, &info, &options, naga::ShaderStage::Fragment, fragment_entry )?;
    let vertex = bindings_rename( &module, &vertex_reflection, &vertex );
    let fragment = bindings_rename( &module, &fragment_reflection, &fragment );

    Ok( WebglGlslSource { vertex, fragment } )
  }

  /// Translates one stage of `module` to GLSL text via naga's GLSL backend.
  fn stage_translate
  (
    module : &naga::Module,
    info : &naga::valid::ModuleInfo,
    options : &glsl::Options,
    shader_stage : naga::ShaderStage,
    entry_point : &str,
  ) -> Result< ( String, glsl::ReflectionInfo ), String >
  {
    let pipeline_options = glsl::PipelineOptions
    {
      shader_stage,
      entry_point : entry_point.to_string(),
      multiview : None
    };
    let mut out = String::new();
    let reflection = glsl::Writer::new( &mut out, module, info, options, &pipeline_options, naga::proc::BoundsCheckPolicies::default() )
    .map_err( | e | format!( "GLSL writer construction failed for {entry_point} :: {e}" ) )?
    .write()
    .map_err( | e | format!( "GLSL translation failed for {entry_point} :: {e}" ) )?;
    Ok( ( out, reflection ) )
  }

  /// Applies both [`uniform_blocks_rename`] and [`texture_uniforms_rename`]
  /// to one stage's GLSL output.
  fn bindings_rename( module : &naga::Module, reflection : &glsl::ReflectionInfo, glsl_source : &str ) -> String
  {
    let glsl_source = uniform_blocks_rename( module, reflection, glsl_source );
    texture_uniforms_rename( module, reflection, &glsl_source )
  }

  /// Renames every uniform block naga generated a name for to
  /// `ub_{group}_{binding}`, read from each global variable's real
  /// `@group`/`@binding` binding — not a guessed pattern, and not limited to
  /// a single block.
  fn uniform_blocks_rename( module : &naga::Module, reflection : &glsl::ReflectionInfo, glsl_source : &str ) -> String
  {
    let mut glsl_source = glsl_source.to_string();
    for ( handle, generated_name ) in &reflection.uniforms
    {
      let Some( binding ) = &module.global_variables[ *handle ].binding else { continue };
      let canonical_name = format!( "ub_{}_{}", binding.group, binding.binding );
      glsl_source = glsl_source.replace( generated_name.as_str(), &canonical_name );
    }
    glsl_source
  }

  /// Renames every combined texture/sampler uniform naga generated a name
  /// for to `tex_{group}_{binding}`, read from the *texture*'s real
  /// `@group`/`@binding` binding. The paired WGSL `sampler` ( when present )
  /// never gets its own GLSL identifier — GLSL ES combines a texture and
  /// its sampler into one `samplerN` uniform — so only the texture's
  /// binding is consulted, matching gpu_hal's WebGL introspection
  /// ( `webgl_bindings_introspect`, `device.rs` ), which looks up texture
  /// uniforms by name and treats sampler bindings as a no-op.
  fn texture_uniforms_rename( module : &naga::Module, reflection : &glsl::ReflectionInfo, glsl_source : &str ) -> String
  {
    let mut glsl_source = glsl_source.to_string();
    for ( generated_name, mapping ) in &reflection.texture_mapping
    {
      let Some( binding ) = &module.global_variables[ mapping.texture ].binding else { continue };
      let canonical_name = format!( "tex_{}_{}", binding.group, binding.binding );
      glsl_source = glsl_source.replace( generated_name.as_str(), &canonical_name );
    }
    glsl_source
  }
}

crate::mod_interface!
{
  own use
  {
    WebglGlslSource,
    wgsl_to_webgl_glsl,
  };
}

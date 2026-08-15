/// Internal namespace.
mod private
{
  use crate::web_sys;
  use wasm_bindgen_futures::JsFuture;

  /// A builder-style struct for creating a `GpuShaderModule`.
  pub struct ShaderModule< 'a >
  {
    /// The source code of the shader, typically written in WGSL.
    code : &'a str,
    /// An optional label for debugging and identification purposes.
    label : Option< &'a str >
  }

  impl< 'a > ShaderModule< 'a > 
  {
    /// Creates a new `ShaderModule` instance with a given shader source code.
    #[ inline ]
    #[ must_use ]
    pub fn new( code : &'a str ) -> Self
    {
      let label = None;

      ShaderModule
      {
        code,
        label
      }
    } 

    /// Sets an optional label for the shader module.
    #[ inline ]
    #[ must_use ]
    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    /// Creates the `GpuShaderModule` using the configured properties.
    #[ inline ]
    #[ must_use ]
    pub fn create( self, device : &web_sys::GpuDevice ) -> web_sys::GpuShaderModule
    {
      let desc = web_sys::GpuShaderModuleDescriptor::new( self.code );

      if let Some( v ) = self.label { desc.set_label( v ); }

      device.create_shader_module( &desc )
    }
  }

  /// A convenience function to create a `GpuShaderModule` with just the code.
  #[ inline ]
  #[ must_use ]
  pub fn create( device : &web_sys::GpuDevice, code : &str ) -> web_sys::GpuShaderModule
  {
    ShaderModule::new( code ).create( device )
  }

  /// One diagnostic message from `GpuShaderModule.getCompilationInfo()`, decoupled from the
  /// raw `web_sys::GpuCompilation*` types so callers never touch the `web_sys_unstable_apis`
  /// cfg or the crate's own compilation-info feature flags directly.
  #[ derive( Debug, Clone ) ]
  pub struct CompilationMessage
  {
    /// Human-readable diagnostic text.
    pub text : String,
    /// Severity of the message.
    pub kind : CompilationMessageKind,
    /// 1-based line number the message refers to.
    pub line : f64,
    /// 1-based column on that line the message refers to.
    pub column : f64
  }

  /// Severity of a `CompilationMessage`, mirroring `web_sys::GpuCompilationMessageType`.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum CompilationMessageKind
  {
    /// The shader failed to compile.
    Error,
    /// The shader compiled, but with a caller-visible caveat.
    Warning,
    /// Informational only.
    Info
  }

  /// Awaits `GpuShaderModule.getCompilationInfo()` and returns its messages as owned data.
  ///
  /// Never fails outright: a rejected promise (the spec allows this only in exceptional host
  /// conditions) yields an empty `Vec` rather than propagating an error, since a shader that
  /// compiled far enough to produce a `GpuShaderModule` at all has no caller-actionable failure
  /// mode here.
  #[ must_use ]
  pub async fn compilation_messages_get( module : &web_sys::GpuShaderModule ) -> Vec< CompilationMessage >
  {
    let info = match JsFuture::from( module.get_compilation_info() ).await
    {
      Ok( info ) => info,
      Err( _ ) => return Vec::new(),
    };

    info.messages().iter().map( | message |
    {
      let kind = match message.type_()
      {
        web_sys::GpuCompilationMessageType::Error => CompilationMessageKind::Error,
        web_sys::GpuCompilationMessageType::Warning => CompilationMessageKind::Warning,
        web_sys::GpuCompilationMessageType::Info => CompilationMessageKind::Info,
        // `GpuCompilationMessageType` is `#[non_exhaustive]` (wasm-bindgen JS string enum) --
        // treat any future/unrecognized severity as blocking rather than silently as benign.
        _ => CompilationMessageKind::Error,
      };

      CompilationMessage
      {
        text : message.message(),
        kind,
        line : message.line_num(),
        column : message.line_pos()
      }
    })
    .collect()
  }

  /// Whether `messages` contains at least one `CompilationMessageKind::Error` -- the signal
  /// that a pipeline rebuild from this shader module should not even be attempted.
  #[ inline ]
  #[ must_use ]
  pub fn has_blocking_error( messages : &[ CompilationMessage ] ) -> bool
  {
    messages.iter().any( | message | message.kind == CompilationMessageKind::Error )
  }
}

crate::mod_interface!
{
  own use
  {
    create,
    compilation_messages_get,
    has_blocking_error
  };
  exposed use
  {
    ShaderModule,
    CompilationMessage,
    CompilationMessageKind
  };
}

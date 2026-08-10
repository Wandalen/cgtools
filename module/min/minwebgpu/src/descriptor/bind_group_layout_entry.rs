/// Internal namespace.
mod private
{
  use crate::*;

  /// Represents a single entry in a WebGPU bind group layout.
  #[ derive( Clone ) ]
  pub struct BindGroupLayoutEntry
  {
    /// The binding number (slot) for this resource in the shader.
    ///
    /// This corresponds to the `@binding(...)` attribute in WGSL (WebGPU Shading Language)
    /// and must be a unique number within the bind group.
    binding : u32,
    /// The visibility of the binding to different shader stages.
    ///
    /// This is a bitmask that specifies which shader stages can access this resource.
    /// For example, a uniform buffer might be visible to both the vertex and fragment stages.
    visibility : u32,
    /// The type of resource being bound.
    ///
    /// This enum specifies the kind of resource, such as a uniform buffer, a sampled
    /// texture, or a sampler. This is a key part of the bind group layout.
    ///
    /// Defaults to `BindingType::Other`, a placeholder. Converting a `BindGroupLayoutEntry`
    /// still carrying that placeholder (i.e. `.ty(..)` was never called) fails with
    /// `error::BindGroupError::TypeNotSet` rather than succeeding with an invalid layout.
    ty : BindingType
  }

  impl BindGroupLayoutEntry
  {
    /// Creates a new `BindGroupLayoutEntry` with default values.
    pub fn new() -> Self
    {
      let binding = 0;
      let visibility = 0;
      let ty = BindingType::Other;

      BindGroupLayoutEntry
      {
        binding,
        visibility,
        ty
      }
    }

    /// Sets the `visibility` to `All`
    pub fn all( self ) -> Self
    {
      self.fragment().compute().vertex()
    }

    /// Sets the visibility of the entry to VERTEX
    pub fn vertex( mut self ) -> Self
    {
      self.visibility |= web_sys::gpu_shader_stage::VERTEX;
      self
    }

    /// Sets the visibility of the entry to FRAGMENT
    pub fn fragment( mut self ) -> Self
    {
      self.visibility |= web_sys::gpu_shader_stage::FRAGMENT;
      self
    }

    /// Sets the visibility of the entry to COMPUTE
    pub fn compute( mut self ) -> Self
    {
      self.visibility |= web_sys::gpu_shader_stage::COMPUTE;
      self
    }
    
    /// Sets the binding value of the entry
    pub fn binding( mut self, binding : u32 ) -> Self
    {
      self.binding = binding;
      self
    }
    
    /// Sets the type of the entry
    pub fn ty( mut self, ty : impl Into< BindingType > ) -> Self
    {
      self.ty = ty.into();
      self
    }
  }

  // Fix(BUG-051): this conversion used to be an infallible `From` that panicked with
  // "The type of the binding entry was not set" whenever `ty` was still `BindingType::Other` —
  // yet `Other` is `BindGroupLayoutEntry::new()`'s own default and is documented (binding_type.rs)
  // as "a placeholder for other or unhandled binding types", i.e. expected, reachable, recoverable
  // input, not an invariant violation. Converted to `TryFrom`, returning
  // `WebGPUError::BindGroupError(BindGroupError::TypeNotSet(binding))` instead of panicking.
  // Root cause: an infallible `From` was used for a conversion that has one documented input
  // variant (`Other`, the struct's own default, reachable simply by never calling `.ty(..)`)
  // with no valid WebGPU representation.
  // Pitfall: a placeholder/default enum variant reachable via ordinary construction is
  // foreseeable caller input, not an invariant violation — the conversion touching it must be
  // fallible (`TryFrom`), never `From` plus a panic on the unhandled arm.
  impl TryFrom< BindGroupLayoutEntry > for web_sys::GpuBindGroupLayoutEntry
  {
    type Error = WebGPUError;

    fn try_from( value: BindGroupLayoutEntry ) -> Result< Self, Self::Error >
    {
      let layout = web_sys::GpuBindGroupLayoutEntry::new( value.binding, value.visibility );

      match &value.ty
      {
        BindingType::Buffer( buffer ) => layout.set_buffer( &buffer ),
        BindingType::Sampler( sampler ) => layout.set_sampler( &sampler ),
        BindingType::Texture( texture ) => layout.set_texture( &texture ),
        BindingType::StorageTexture( texture ) => layout.set_storage_texture( &texture ),
        BindingType::ExternalTexture( texture ) => layout.set_external_texture( &texture ),
        BindingType::Other => return Err( error::BindGroupError::TypeNotSet( value.binding ).into() )
      }

      Ok( layout )
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    BindGroupLayoutEntry
  };
}

/// Internal namespace.
mod private
{
  use crate::{ web_sys, VertexAttribute, Into, VertexBufferLayout, BindGroupDescriptor, RenderPassDescriptor, RenderPipelineDescriptor, SamplerDescriptor, TextureDescriptor, BindGroupLayoutDescriptor, PipelineLayoutDescriptor, BufferDescriptor, BindGroupEntry, BufferBinding, VertexState, FragmentState, DepthStencilState, StencilFaceState, MultiSampleState, PrimitiveState, ColorTargetState, BlendComponent, BlendState, ColorAttachment, DepthStencilAttachment };

  /// A generic trait for converting a type into its WebGPU equivalent.
  pub trait AsWeb< T >
  {
    /// Converts the implementing type into the target WebGPU type.
    fn to_web( self ) -> T;
  }

  macro_rules! impl_to_web 
  {
    ( $local:ty, $web:ident ) => 
    {
        impl AsWeb< web_sys::$web > for $local
        {
            #[ inline ]
            fn to_web( self ) -> web_sys::$web
            {
                self.into()
            }
        }
    };
  }

  // Layout
  impl_to_web!( VertexAttribute, GpuVertexAttribute );
  impl_to_web!( VertexBufferLayout, GpuVertexBufferLayout );

  // Descriptor
  impl_to_web!( BindGroupDescriptor< '_ >, GpuBindGroupDescriptor );
  impl_to_web!( RenderPassDescriptor< '_ >, GpuRenderPassDescriptor );
  impl_to_web!( RenderPipelineDescriptor< '_ >, GpuRenderPipelineDescriptor );
  impl_to_web!( SamplerDescriptor< '_ >, GpuSamplerDescriptor );
  impl_to_web!( TextureDescriptor< '_ >, GpuTextureDescriptor );
  // Fix(BUG-051): removed `impl_to_web!( BindGroupLayoutEntry, GpuBindGroupLayoutEntry );` —
  // `BindGroupLayoutEntry` intentionally has no `AsWeb` impl: its conversion to
  // `web_sys::GpuBindGroupLayoutEntry` is fallible (fails with `error::BindGroupError::TypeNotSet`
  // when `.ty(..)` was never called), and `AsWeb::to_web` is infallible by design. Use
  // `TryInto::try_into` / `TryFrom::try_from` instead.
  // Root cause: this macro line assumed `BindGroupLayoutEntry`'s conversion was, and would
  // remain, infallible — `AsWeb::to_web` has no `Result` in its signature to carry a failure.
  // Pitfall: `impl_to_web!` unconditionally assumes `Into`/infallibility for whatever type it's
  // instantiated with; once a type's conversion becomes fallible (`TryFrom`), its `impl_to_web!`
  // line must be removed, not left to silently fail to compile (or worse, silently compile
  // against a stale blanket `Into` if one still existed).
  impl_to_web!( BindGroupLayoutDescriptor, GpuBindGroupLayoutDescriptor );
  impl_to_web!( PipelineLayoutDescriptor< '_ >, GpuPipelineLayoutDescriptor );
  impl_to_web!( BufferDescriptor< '_ >, GpuBufferDescriptor );

  // Bind group entry
  impl_to_web!( BindGroupEntry, GpuBindGroupEntry );
  impl_to_web!( BufferBinding< '_ >, GpuBufferBinding );

  // State
  impl_to_web!( VertexState< '_ >, GpuVertexState );
  impl_to_web!( FragmentState< '_ >, GpuFragmentState );
  impl_to_web!( DepthStencilState, GpuDepthStencilState );
  impl_to_web!( StencilFaceState, GpuStencilFaceState );
  impl_to_web!( MultiSampleState, GpuMultisampleState );
  impl_to_web!( PrimitiveState, GpuPrimitiveState );
  impl_to_web!( ColorTargetState, GpuColorTargetState );
  impl_to_web!( BlendComponent, GpuBlendComponent );
  impl_to_web!( BlendState, GpuBlendState );

  // Render pass
  impl_to_web!( ColorAttachment< '_ >, GpuRenderPassColorAttachment );
  impl_to_web!( DepthStencilAttachment< '_ >, GpuRenderPassDepthStencilAttachment );

}

crate::mod_interface!
{
  exposed use
  {
    AsWeb
  };
}

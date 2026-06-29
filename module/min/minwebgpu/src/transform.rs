/// Internal namespace.
mod private
{
  use crate::*;
  // Shadow the glob-imported collection_tools `Vec` (printed under its `Dlist` alias)
  // with std `Vec`, so sequence-conversion errors name `Vec`, not `Dlist`.
  use std::vec::Vec;

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
  impl_to_web!( BindGroupLayoutEntry, GpuBindGroupLayoutEntry );
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

  /// Wraps each element of `items` in a `js_sys::JsOption`, producing the
  /// nullable-sequence representation web-sys expects for `&[ JsOption< T > ]`
  /// setters (e.g. render-pass color attachments, vertex buffers, fragment targets).
  pub fn js_option_vec< T >( items : Vec< T > ) -> Vec< js_sys::JsOption< T > >
  where
    T : js_sys::JsGeneric,
  {
    items.into_iter().map( js_sys::JsOption::wrap ).collect()
  }

  /// Converts an iterator of numeric values into a `Vec< js_sys::Number >`, the
  /// representation web-sys expects for `&[ js_sys::Number ]` setters (e.g. texture
  /// size, clear values).
  pub fn number_vec< I >( items : I ) -> Vec< js_sys::Number >
  where
    I : IntoIterator,
    I::Item : Into< js_sys::Number >,
  {
    items.into_iter().map( Into::into ).collect()
  }

  /// Converts an iterator of JS-string-enum values (e.g. `GpuTextureFormat`) into the
  /// `Vec< js_sys::JsString >` representation web-sys expects for string-enum sequence
  /// setters (e.g. texture `view_formats`). Each item is a wasm-bindgen string enum, so
  /// `Into< JsValue >` yields its string representation.
  pub fn js_string_vec< I >( items : I ) -> Vec< js_sys::JsString >
  where
    I : IntoIterator,
    I::Item : Into< wasm_bindgen::JsValue >,
  {
    items.into_iter()
    .map( | v | wasm_bindgen::JsCast::unchecked_into( v.into() ) )
    .collect()
  }

}

crate::mod_interface!
{
  exposed use
  {
    AsWeb,
    js_option_vec,
    number_vec,
    js_string_vec
  };
}

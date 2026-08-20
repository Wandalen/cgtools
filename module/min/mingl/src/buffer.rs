/// Internal namespace.
mod private
{
  use crate::{ VectorDataType, mem };

  /// Whether a vertex attribute advances once per vertex or once per instance.
  ///
  /// WebGPU and core Vulkan only support this binary switch ( `GPUVertexStepMode` /
  /// `VkVertexInputRate` ) — neither has a numeric divisor. WebGL2's `vertexAttribDivisor` supports
  /// an arbitrary divisor on top of this switch; that WebGL-only capability is modeled separately by
  /// `VertexBufferLayout::divisor`, not folded into this enum, so the type stays meaningful on every
  /// backend.
  #[ derive( Debug, Default, Clone, Copy, PartialEq, Eq ) ]
  pub enum StepMode
  {
    /// Advances once per vertex.
    #[ default ]
    Vertex,
    /// Advances once per instance.
    Instance,
  }

  /// Describes one vertex attribute's binding within a buffer: where it binds and what shape its
  /// data has. Buffer-wide concerns ( stride, step mode, divisor ) live on `VertexBufferLayout`
  /// instead, matching the two-level split every real cross-backend GPU API ( WebGPU, Vulkan ) uses.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct VertexAttribute
  {
    /// The shader attribute location this attribute binds to.
    pub location : u32,
    /// The vector data type of the attribute.
    pub vector : VectorDataType,
    /// The offset of this attribute within one buffer element.
    pub offset : i32,
  }

  impl VertexAttribute
  {
    /// Creates a new `VertexAttribute`.
    #[ inline ]
    #[ must_use ]
    pub fn new( location : u32, vector : VectorDataType, offset : i32 ) -> Self
    {
      Self { location, vector, offset }
    }
  }

  /// Describes the full layout of one vertex buffer: its stride, step mode, and the attributes
  /// bound within it.
  #[ derive( Debug, Clone, Default ) ]
  pub struct VertexBufferLayout
  {
    /// The stride between consecutive elements, in the same units as each attribute's `offset`.
    pub stride : i32,
    /// Whether attributes in this buffer advance per-vertex or per-instance.
    pub step_mode : StepMode,
    /// WebGL-only instancing divisor ( `vertexAttribDivisor` ). Meaningful only when `step_mode` is
    /// `Instance` and only on backends that support an arbitrary divisor ( WebGL2 ); WebGPU/Vulkan
    /// only honor `step_mode`'s binary switch and ignore this field.
    ///
    /// A divisor of 0 indicates that each vertex has its own unique attribute value.
    /// A divisor of 1 means that the entire primitive shares the same attribute value.
    /// A divisor of 2 or more specifies that the attribute value is shared across multiple primitives.
    pub divisor : usize,
    /// The attributes bound within this buffer.
    pub attributes : Vec< VertexAttribute >,
  }

  impl VertexBufferLayout
  {
    /// Creates a new, empty `VertexBufferLayout`.
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Sets the stride.
    #[ inline ]
    #[ must_use ]
    pub fn stride( mut self, src : i32 ) -> Self
    {
      self.stride = src;
      self
    }

    /// Sets the step mode.
    #[ inline ]
    #[ must_use ]
    pub fn step_mode( mut self, src : StepMode ) -> Self
    {
      self.step_mode = src;
      self
    }

    /// Sets the WebGL-only instancing divisor.
    #[ inline ]
    #[ must_use ]
    pub fn divisor( mut self, src : usize ) -> Self
    {
      self.divisor = src;
      self
    }

    /// Appends an attribute to this layout.
    #[ inline ]
    #[ must_use ]
    pub fn attribute( mut self, src : VertexAttribute ) -> Self
    {
      self.attributes.push( src );
      self
    }

    /// Builds a layout from a `Pod` type's own `Attribute::describe()`, with the given stride.
    #[ inline ]
    #[ must_use ]
    pub fn from_attribute< T : Attribute >( stride : i32 ) -> Self
    {
      Self
      {
        stride,
        attributes : T::describe(),
        ..Self::default()
      }
    }
  }

  /// Trait for `Pod` types that can declaratively describe their own vertex-attribute layout —
  /// implemented once per vertex struct, called as `Vertex::describe()`.
  pub trait Attribute : mem::Pod
  {
    /// Describes the attribute bindings for the type.
    fn describe() -> Vec< VertexAttribute >;
  }

  impl Attribute for ()
  {
    /// Returns an empty vector as there are no attributes.
    fn describe() -> Vec< VertexAttribute >
    {
      Vec::new()
    }
  }

}

crate::mod_interface!
{

  orphan use
  {
    StepMode,
    VertexAttribute,
    VertexBufferLayout,
    Attribute,
  };

}

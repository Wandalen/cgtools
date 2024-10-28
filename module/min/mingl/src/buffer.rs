/// Internal namespace.
mod private
{
  // use crate::*;

//   pub struct BufferDescriptor
//   {
//     pub vector : VectorDataType,
//     pub offset : i32,
//     pub stride : i32,
//     pub divisor : usize,
//   }
//
//   impl BufferDescriptor
//   {
//
//     pub fn vector( mut self, src : VectorDataType ) -> Self
//     {
//       self.vector = src;
//       self
//     }
//
//     pub fn offset( mut self, src : i32 ) -> Self
//     {
//       self.offset = src;
//       self
//     }
//
//     pub fn stride( mut self, src : i32 ) -> Self
//     {
//       self.stride = src;
//       self
//     }
//
//     pub fn divisor( mut self, src : usize ) -> Self
//     {
//       self.divisor = src;
//       self
//     }
//
//     pub fn new< I : IntoVectorDataType >() -> Self
//     {
//       let vector = I::into_vector_data_type();
//       let offset = 0;
//       let stride = 0;
//       let divisor = 0;
//       Self
//       {
//         vector,
//         offset,
//         stride,
//         divisor,
//       }
//     }
//
//   }

}

crate::mod_interface!
{

  orphan use
  {
    // BufferDescriptor,
  };

}

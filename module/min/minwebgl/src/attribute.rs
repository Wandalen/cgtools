/// Internal namespace.
mod private
{
//   use crate::*;
//
//   use data_type::
//   {
//     DataType,
//     VectorDataType
//   };
//
//   /// Represents a binding between a shader variable and its data type.
//   #[ derive( Debug, PartialEq, Clone ) ]
//   pub struct AttributeDescription
//   {
//     /// The name of the shader variable.
//     pub variable_name : String,
//     /// The data type and size of the attribute.
//     pub kind : VectorDataType,
//   }
//
//   impl AttributeDescription
//   {
//     /// Creates a new attribute binding with the specified name, data type, and size.
//     pub fn new( name : &str, scalar : DataType, size : i32 ) -> Self
//     {
//       AttributeDescription
//       {
//         variable_name : name.to_string(),
//         kind : VectorDataType::new( scalar, size ),
//       }
//     }
//   }
//
//   /// Trait for types that can be used as attributes in shaders.
//   pub trait Attribute : mem::Pod
//   {
//     /// Describes the attribute bindings for the type.
//     fn describe() -> Vec< AttributeDescription >;
//   }
//
//   impl Attribute for ()
//   {
//     /// Returns an empty vector as there are no attributes.
//     fn describe() -> Vec< AttributeDescription >
//     {
//       Vec::new()
//     }
//   }

}

crate::mod_interface!
{

  orphan use
  {
    // AttributeDescription,
    // Attribute,
  };

}

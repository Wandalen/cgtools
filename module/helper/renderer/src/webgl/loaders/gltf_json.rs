//! Reading numbers out of the extension JSON the `gltf` crate has no typed accessor for. Both
//! readers of such JSON use these : `gltf_extensions` ( material extensions ) and `meshopt`
//! ( buffer view extensions ).

use gltf::json::Value;

/// `value` as a `usize`, when it is a non-negative integer that fits one.
pub( crate ) fn usize_read( value : &Value ) -> Option< usize >
{
  value.as_u64().and_then( | v | usize::try_from( v ).ok() )
}

/// The number under `key` in `json` as an `f32`, or `default` when it is absent or not a number.
pub( crate ) fn f32_get( json : &Value, key : &str, default : f32 ) -> f32
{
  json.get( key ).and_then( Value::as_f64 ).map_or( default, | v | v as f32 )
}

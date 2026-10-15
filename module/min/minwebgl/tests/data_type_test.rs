//! Native tests for the `data_type` pure-logic layer — the `DataType` ↔
//! `Const< DataType >` conversions, exercised through the crate's public API only.
//! (`Const` cannot be constructed externally; `TryFrom< DataType >` is the sole
//! public entry, so the roundtrip below covers both conversion directions.)

use minwebgl::data_type::{ Const, DataType };

/// The 7 convertible scalar types and the WebGL2 constants they must map to
/// (`BYTE`, `UNSIGNED_BYTE`, `SHORT`, `UNSIGNED_SHORT`, `INT`, `UNSIGNED_INT`, `FLOAT`).
const EXPECTED : [ ( DataType, u32 ) ; 7 ] =
[
  ( DataType::I8, 0x1400 ),
  ( DataType::U8, 0x1401 ),
  ( DataType::I16, 0x1402 ),
  ( DataType::U16, 0x1403 ),
  ( DataType::I32, 0x1404 ),
  ( DataType::U32, 0x1405 ),
  ( DataType::F32, 0x1406 ),
];

#[ test ]
fn data_type_to_const_pins_webgl_constants()
{
  for ( data_type, expected ) in EXPECTED
  {
    let converted = Const::< DataType >::try_from( data_type )
    .unwrap_or_else( | error | panic!( "{data_type:?} must convert: {error}" ) );
    assert_eq!( *converted, expected, "{data_type:?} must map to {expected:#06x}" );
  }
}

#[ test ]
fn const_to_data_type_roundtrips()
{
  for ( data_type, _ ) in EXPECTED
  {
    let converted = Const::< DataType >::try_from( data_type ).unwrap();
    let back = DataType::try_from( converted )
    .unwrap_or_else( | error | panic!( "{data_type:?} must roundtrip: {error}" ) );
    assert_eq!( back, data_type );
  }
}

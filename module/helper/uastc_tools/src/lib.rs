//! Vendored UASTC (Basis Universal) block transcoder.
//!
//! See `readme.md` for provenance: this is a subset of `basisu_rs` by Jakub Valtar,
//! MIT OR Apache-2.0, vendored because upstream is unpublished. The `.basis`
//! container and ETC1S/BasisLZ code paths are deliberately **not** vendored — cgtools
//! reads UASTC out of KTX2 containers via the `ktx2` crate, so the container layer
//! upstream provides is redundant, and ETC1S is out of scope.
#![ doc( html_root_url = "https://docs.rs/uastc_tools/latest/uastc_tools/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ forbid( unsafe_code ) ]
#![ no_std ]

// The BC7 and ETC transcoders use inherent `f32` methods ( `powi`, `round`, `sqrt` ),
// which live in `std`, not `core`. Linking `std` brings them into scope. Kept behind a
// feature, as upstream does, so the crate stays usable in a `no_std` build.
#[ cfg( feature = "std" ) ]
extern crate std;

extern crate alloc;

// ---------------------------------------------------------------------------------------
// VENDORED THIRD-PARTY CODE -- lints relaxed, here and only here.
//
// The five modules below are kept byte-for-byte as upstream wrote them ( see readme.md:
// `basisu_rs` @ 60e1bcb ). They are deliberately not reformatted or re-idiomed to cgtools
// conventions: the whole value of recording the upstream commit is being able to `diff`
// against it, and rewriting 3,500 lines of a bit-exact-validated block decoder would
// destroy that for no functional gain and considerable risk.
//
// The relaxations are attached to the module declarations rather than to the crate, so
// that this file -- the only hand-written, cgtools-owned code here -- stays fully linted.
//
// * `dead_code`: upstream is a complete Basis Universal implementation. cgtools calls only
//   the UASTC block entry points, so its `.basis` container reader, ETC1S paths and
//   assorted helpers are genuinely unreachable from our surface. Deleting them would be a
//   bigger, riskier diff from upstream than tolerating them.
// * `clippy::pedantic`, and the individual restriction lints the workspace turns on
//   ( `min_ident_chars`, `else_if_without_else`, `exhaustive_enums`, ... ): style
//   judgements about code we have chosen not to restyle. Suppressing them is the direct
//   consequence of the byte-for-byte decision above; treating them as actionable would
//   contradict it.
//
// Nothing here is silenced for *correctness*. `unsafe_code` stays forbidden crate-wide,
// and the transcoders remain validated bit-exactly against KTX-Software ( readme.md, T1 ).
// ---------------------------------------------------------------------------------------

/// Declares a vendored module with the workspace lints relaxed. See the comment block above.
macro_rules! vendored
{
  ( $( $name : ident ),* $(,)? ) =>
  {
    $(
      // Upstream is a complete Basis Universal implementation; we call only part of it.
      #[ allow( dead_code ) ]
      // Style-only, and all consequences of not restyling vendored source.
      #[ allow( elided_lifetimes_in_paths ) ]
      #[ allow( clippy::pedantic ) ]
      #[ allow( clippy::get_first, clippy::needless_range_loop ) ]
      #[ allow( clippy::min_ident_chars, clippy::else_if_without_else ) ]
      #[ allow( clippy::exhaustive_enums, clippy::exhaustive_structs ) ]
      #[ allow( clippy::missing_inline_in_public_items, clippy::wildcard_imports ) ]
      #[ allow( clippy::std_instead_of_core, clippy::std_instead_of_alloc ) ]
      mod $name;
    )*
  };
}

vendored!( bitreader, bitwriter, color, target_formats, uastc );

use color::Color32;
use uastc::{ ASTC_BLOCK_SIZE, BC7_BLOCK_SIZE, ETC1_BLOCK_SIZE, ETC2_BLOCK_SIZE, UASTC_BLOCK_SIZE };

/// Error returned when a block is malformed.
type Error = alloc::string::String;
/// Result of a block decode / transcode.
type Result< T > = core::result::Result< T, Error >;

/// Decode a UASTC block to 16 RGBA texels, in block-local row-major order.
///
/// Each `u32` is packed little-endian as `[ r, g, b, a ]` — note this is **RGBA**, not
/// the BGRA that some other block decoders (e.g. `texture2ddecoder`) emit.
///
/// Used for the uncompressed fallback, when the device supports no compressed format.
///
/// # Errors
/// Returns `Err` if the block is malformed.
#[ inline ]
pub fn unpack_uastc_block_to_rgba( data : [ u8; UASTC_BLOCK_SIZE ] ) -> Result< [ u32; 16 ] >
{
  uastc::decode_block_to_rgba( data ).map( | texels | texels.map( Color32::to_rgba_u32 ) )
}

/// Transcode a UASTC block to an ASTC 4x4 block.
///
/// # Errors
/// Returns `Err` if the block is malformed.
#[ inline ]
pub fn transcode_uastc_block_to_astc( data : [ u8; UASTC_BLOCK_SIZE ] ) -> Result< [ u8; ASTC_BLOCK_SIZE ] >
{
  target_formats::astc::convert_block_from_uastc( data )
}

/// Transcode a UASTC block to a BC7 mode-5 block.
///
/// # Errors
/// Returns `Err` if the block is malformed.
#[ inline ]
pub fn transcode_uastc_block_to_bc7( data : [ u8; UASTC_BLOCK_SIZE ] ) -> Result< [ u8; BC7_BLOCK_SIZE ] >
{
  target_formats::bc7::convert_block_from_uastc( data )
}

/// Transcode a UASTC block to an ETC1 block.
///
/// # Errors
/// Returns `Err` if the block is malformed.
#[ inline ]
pub fn transcode_uastc_block_to_etc1( data : [ u8; UASTC_BLOCK_SIZE ] ) -> Result< [ u8; ETC1_BLOCK_SIZE ] >
{
  target_formats::etc::convert_etc1_block_from_uastc( data )
}

/// Transcode a UASTC block to an ETC2 RGBA block.
///
/// # Errors
/// Returns `Err` if the block is malformed.
#[ inline ]
pub fn transcode_uastc_block_to_etc2( data : [ u8; UASTC_BLOCK_SIZE ] ) -> Result< [ u8; ETC2_BLOCK_SIZE ] >
{
  target_formats::etc::convert_etc2_block_from_uastc( data )
}

/// Bit mask of `$size` low bits. Internal to the vendored decoder.
#[ doc( hidden ) ]
#[ macro_export ]
macro_rules! mask
{
  ( $size:expr ) =>
  {
    !( !( $size ^ $size ) ).checked_shl( $size as u32 ).unwrap_or( 0 )
  };
}

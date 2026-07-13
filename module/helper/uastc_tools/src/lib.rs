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

// The modules below `lib.rs` are VENDORED THIRD-PARTY CODE, kept byte-for-byte as
// upstream wrote it ( see readme.md: `basisu_rs` @ 60e1bcb ). They are deliberately not
// reformatted or re-idiomed to cgtools conventions: the whole value of recording the
// upstream commit is being able to `diff` against it, and rewriting 3,500 lines of a
// bit-exact-validated block decoder would destroy that for no functional gain.
// The workspace lints are therefore relaxed here, and here only. This crate's own
// surface -- the entry points in this file -- does follow the repo conventions.
#![ allow( elided_lifetimes_in_paths ) ]

// The BC7 and ETC transcoders use inherent `f32` methods ( `powi`, `round`, `sqrt` ),
// which live in `std`, not `core`. Linking `std` brings them into scope. Kept behind a
// feature, as upstream does, so the crate stays usable in a `no_std` build.
#[ cfg( feature = "std" ) ]
extern crate std;

extern crate alloc;

mod bitreader;
mod bitwriter;
mod color;
mod target_formats;
mod uastc;

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
pub fn unpack_uastc_block_to_rgba( data : [ u8; UASTC_BLOCK_SIZE ] ) -> Result< [ u32; 16 ] >
{
  uastc::decode_block_to_rgba( data ).map( | b | b.map( Color32::to_rgba_u32 ) )
}

/// Transcode a UASTC block to an ASTC 4x4 block.
///
/// # Errors
/// Returns `Err` if the block is malformed.
pub fn transcode_uastc_block_to_astc( data : [ u8; UASTC_BLOCK_SIZE ] ) -> Result< [ u8; ASTC_BLOCK_SIZE ] >
{
  target_formats::astc::convert_block_from_uastc( data )
}

/// Transcode a UASTC block to a BC7 mode-5 block.
///
/// # Errors
/// Returns `Err` if the block is malformed.
pub fn transcode_uastc_block_to_bc7( data : [ u8; UASTC_BLOCK_SIZE ] ) -> Result< [ u8; BC7_BLOCK_SIZE ] >
{
  target_formats::bc7::convert_block_from_uastc( data )
}

/// Transcode a UASTC block to an ETC1 block.
///
/// # Errors
/// Returns `Err` if the block is malformed.
pub fn transcode_uastc_block_to_etc1( data : [ u8; UASTC_BLOCK_SIZE ] ) -> Result< [ u8; ETC1_BLOCK_SIZE ] >
{
  target_formats::etc::convert_etc1_block_from_uastc( data )
}

/// Transcode a UASTC block to an ETC2 RGBA block.
///
/// # Errors
/// Returns `Err` if the block is malformed.
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

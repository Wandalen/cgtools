//! Tests for `minwebgl::texture::compressed`.
//!
//! Format *selection* is pure logic over a set of booleans, so it is separated from the
//! `getExtension` calls that produce those booleans ( `Support::query`, which needs a real GPU
//! context and is therefore not covered here ). That split is what makes these tests possible
//! at all: they run natively under `cargo nextest`, with no browser and no WebGL.
//!
//! What they pin down:
//!
//! * the fidelity ordering of `Support::best` -- including the overlaps a real device rarely
//!   produces but that a driver could, which is precisely where a silent regression would hide;
//! * that `RGBA8` is reachable *only* as a fallback, never preferred over a compressed format;
//! * the size arithmetic `compressedTexImage2D` validates against. WebGL rejects a byte count
//!   that disagrees with `internalformat` + dimensions, so an off-by-one in the block rounding
//!   is a hard upload failure, not a visual artifact.

#![ cfg( feature = "constants" ) ]

use minwebgl::texture::compressed::{ ColorSpace, Format, Support };

/// Builds a `Support` from the three selectable flags, in `( astc, bptc, etc2 )` order.
///
/// `s3tc` is left off: it is never selectable, and only participates in the emulation
/// heuristic, which the tests that care about it construct explicitly.
const fn support( astc : bool, bptc : bool, etc2 : bool ) -> Support
{
  Support { astc, bptc, etc2, s3tc : false }
}

/// Each family, alone, selects its own format.
#[ test ]
fn best_selects_each_family_in_isolation()
{
  assert_eq!( support( true,  false, false ).best(), Format::Astc4x4 );
  assert_eq!( support( false, true,  false ).best(), Format::Bc7 );
  assert_eq!( support( false, false, true  ).best(), Format::Etc2Rgba );
}

/// S3TC alone is not a usable target -- UASTC has no transcode path into it.
#[ test ]
fn s3tc_alone_is_not_selectable()
{
  let s3tc_only = Support { astc : false, bptc : false, etc2 : false, s3tc : true };
  assert_eq!( s3tc_only.best(), Format::Rgba8 );
}

/// A context with no compressed-texture extension at all still gets a usable format.
#[ test ]
fn best_falls_back_to_rgba8_when_nothing_is_supported()
{
  assert_eq!( support( false, false, false ).best(), Format::Rgba8 );
  assert_eq!( Support::default().best(), Format::Rgba8 );
}

/// The fidelity ordering, exercised on every overlap.
///
/// ASTC beats everything because UASTC is a subset of it; BC7 beats ETC2 because ETC2 cannot
/// represent the endpoint precision UASTC carries. Each assertion below is a case where a
/// naive `if bptc { .. } else if astc { .. }` would silently pick the lossier target.
#[ test ]
fn best_prefers_higher_fidelity_targets_on_overlap()
{
  assert_eq!( support( true,  true,  false ).best(), Format::Astc4x4 );
  assert_eq!( support( true,  false, true  ).best(), Format::Astc4x4 );
  assert_eq!( support( true,  true,  true  ).best(), Format::Astc4x4 );
  assert_eq!( support( false, true,  true  ).best(), Format::Bc7 );
}

/// `best` never returns the uncompressed fallback when a compressed format is available --
/// the whole point of the transcode is to keep the texture compressed in VRAM.
#[ test ]
fn best_never_falls_back_when_any_family_is_supported()
{
  for astc in [ false, true ]
  {
    for bptc in [ false, true ]
    {
      for etc2 in [ false, true ]
      {
        let chosen = support( astc, bptc, etc2 ).best();
        let any_supported = astc || bptc || etc2;
        assert_eq!
        (
          chosen == Format::Rgba8,
          !any_supported,
          "( astc: {astc}, bptc: {bptc}, etc2: {etc2} ) chose {chosen:?}"
        );
      }
    }
  }
}

/// The Mesa case the guard exists for: a Linux desktop context advertising *every* family.
///
/// No real GPU implements the mobile and desktop families both, so this can only be a driver
/// saying yes to everything -- and taking it at its word would select ASTC, which Mesa then
/// software-decompresses on the main thread. The guard must drop ASTC and ETC2 and leave BC7,
/// which such hardware genuinely has.
#[ test ]
fn emulation_guard_drops_astc_and_etc2_when_every_family_is_advertised_on_linux()
{
  let everything = Support { astc : true, bptc : true, etc2 : true, s3tc : true };

  assert_eq!( everything.best(), Format::Astc4x4, "unguarded, ASTC would win" );

  let guarded = everything.without_emulated( true );
  assert!( !guarded.astc, "ASTC should be dropped as emulated" );
  assert!( !guarded.etc2, "ETC2 should be dropped as emulated" );
  assert!( guarded.bptc, "BPTC is real on this hardware and must survive" );
  assert_eq!( guarded.best(), Format::Bc7 );
}

/// Genuine ASTC hardware on Linux must **not** be caught by the guard.
///
/// This is the false positive that a platform-only heuristic would produce, and it would be a
/// self-inflicted quality loss: an ASTC-capable Linux GPU advertises ASTC and ETC but not the
/// desktop families, so the "advertises everything" half of the test fails and ASTC survives.
#[ test ]
fn emulation_guard_spares_genuine_astc_hardware_on_linux()
{
  let real_mobile_gpu = Support { astc : true, bptc : false, etc2 : true, s3tc : false };

  let guarded = real_mobile_gpu.without_emulated( true );
  assert_eq!( guarded, real_mobile_gpu, "nothing should have been dropped" );
  assert_eq!( guarded.best(), Format::Astc4x4 );
}

/// Off a Mesa-like platform the guard is inert, even for a context advertising everything.
#[ test ]
fn emulation_guard_is_inert_off_linux()
{
  let everything = Support { astc : true, bptc : true, etc2 : true, s3tc : true };

  assert_eq!( everything.without_emulated( false ), everything );
  assert_eq!( everything.without_emulated( false ).best(), Format::Astc4x4 );
}

/// The guard never *adds* support, on any input -- it can only take away.
#[ test ]
fn emulation_guard_only_removes_support()
{
  for astc in [ false, true ]
  {
    for bptc in [ false, true ]
    {
      for etc2 in [ false, true ]
      {
        for s3tc in [ false, true ]
        {
          let advertised = Support { astc, bptc, etc2, s3tc };

          for mesa_like in [ false, true ]
          {
            let guarded = advertised.without_emulated( mesa_like );
            assert!( !guarded.astc || advertised.astc );
            assert!( !guarded.bptc || advertised.bptc );
            assert!( !guarded.etc2 || advertised.etc2 );
            assert!( !guarded.s3tc || advertised.s3tc );
            // BPTC is never emulated, so it must pass through untouched.
            assert_eq!( guarded.bptc, advertised.bptc );
          }
        }
      }
    }
  }
}

/// Every compressed format is 4x4 with a 16-byte block -- that uniformity is what lets UASTC,
/// itself a 128-bit 4x4 encoding, transcode into any of them.
#[ test ]
fn compressed_formats_are_16_byte_4x4_blocks()
{
  for format in [ Format::Astc4x4, Format::Bc7, Format::Etc2Rgba ]
  {
    assert!( format.is_compressed(), "{format:?} should be compressed" );
    assert_eq!( format.block_bytes(), Some( 16 ), "{format:?}" );
    assert!( format.extension_name().is_some(), "{format:?} needs an extension" );
  }

  assert!( !Format::Rgba8.is_compressed() );
  assert_eq!( Format::Rgba8.block_bytes(), None );
  assert_eq!( Format::Rgba8.extension_name(), None );
}

/// Level sizes for block-aligned dimensions: a 1024x1024 level is 256x256 blocks.
#[ test ]
fn level_size_for_block_aligned_dimensions()
{
  assert_eq!( Format::Bc7.level_size( 1024, 1024 ), 256 * 256 * 16 );
  assert_eq!( Format::Astc4x4.level_size( 4, 4 ), 16 );
  assert_eq!( Format::Etc2Rgba.level_size( 8, 4 ), 2 * 16 );

  // Uncompressed is plain width * height * RGBA.
  assert_eq!( Format::Rgba8.level_size( 1024, 1024 ), 1024 * 1024 * 4 );
}

/// Dimensions that are not a multiple of 4 round **up** to whole blocks.
///
/// This is the case that matters in practice: the tail of a mip chain ( 2x2, 1x1 ) is never
/// block-aligned, so every mipmapped compressed texture hits it. Rounding down, or computing
/// `w * h / 16 * block`, produces a byte count WebGL rejects outright.
#[ test ]
fn level_size_rounds_partial_blocks_up()
{
  // A single, partially-filled block.
  assert_eq!( Format::Bc7.level_size( 1, 1 ), 16 );
  assert_eq!( Format::Bc7.level_size( 2, 2 ), 16 );
  assert_eq!( Format::Bc7.level_size( 3, 3 ), 16 );
  assert_eq!( Format::Bc7.level_size( 4, 4 ), 16 );

  // Crossing into a second block in each axis.
  assert_eq!( Format::Bc7.level_size( 5, 4 ), 2 * 16 );
  assert_eq!( Format::Bc7.level_size( 4, 5 ), 2 * 16 );
  assert_eq!( Format::Bc7.level_size( 5, 5 ), 4 * 16 );

  // A realistic non-power-of-two texture: 100x100 -> 25x25 blocks exactly.
  assert_eq!( Format::Bc7.level_size( 100, 100 ), 25 * 25 * 16 );
  // 99x99 -> still 25x25 blocks, the last one only partially covered.
  assert_eq!( Format::Bc7.level_size( 99, 99 ), 25 * 25 * 16 );

  // A zero-sized level costs nothing rather than one block.
  assert_eq!( Format::Bc7.level_size( 0, 0 ), 0 );
}

/// The linear and sRGB constants are distinct for every format.
///
/// Confusing the two is the single most likely way to get a visibly-wrong-but-not-broken
/// result -- the renderer linearises in the shader, so uploading an sRGB internal format would
/// decode twice and darken the image. Pinning the exact enum values keeps a typo in the
/// constant table from becoming that bug.
#[ test ]
fn internal_format_distinguishes_color_spaces()
{
  for format in [ Format::Astc4x4, Format::Bc7, Format::Etc2Rgba, Format::Rgba8 ]
  {
    assert_ne!
    (
      format.internal_format( ColorSpace::Linear ),
      format.internal_format( ColorSpace::Srgb ),
      "{format:?} uses the same constant for linear and sRGB"
    );
  }

  // Values are fixed by the specs; spell them out so a transcription slip is caught here and
  // not as an INVALID_ENUM at upload time.
  assert_eq!( Format::Astc4x4.internal_format( ColorSpace::Linear ), 0x93B0 );
  assert_eq!( Format::Astc4x4.internal_format( ColorSpace::Srgb ),   0x93D0 );
  assert_eq!( Format::Bc7.internal_format( ColorSpace::Linear ),     0x8E8C );
  assert_eq!( Format::Bc7.internal_format( ColorSpace::Srgb ),       0x8E8D );
  assert_eq!( Format::Etc2Rgba.internal_format( ColorSpace::Linear ), 0x9278 );
  assert_eq!( Format::Etc2Rgba.internal_format( ColorSpace::Srgb ),   0x9279 );
}

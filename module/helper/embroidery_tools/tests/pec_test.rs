//! PEC format tests — decoding the reference sample in `test_files/` and a
//! write→read roundtrip preserving stitches and thread selection. Public API only.

use std::io::Cursor;
use embroidery_tools::embroidery_file::EmbroideryFile;
use embroidery_tools::error::EmbroideryError;
use embroidery_tools::format::pec;
use embroidery_tools::stitch_instruction::{ Instruction, Stitch };
use embroidery_tools::thread::Thread;

#[ test ]
fn read_sample_stitches_match_reference_decoder()
{
  let emb = pec::file_read( "test_files/read_sample.pec" ).unwrap();
  let stitches = emb.stitches();

  // These instructions should match instructions when reading with pyembroidery.
  assert_eq!( stitches[ 0 ], Stitch { x : 10, y : 20, instruction : Instruction::Jump } );
  assert_eq!( stitches[ 1 ], Stitch { x : 10, y : 20, instruction : Instruction::Stitch } );

  assert_eq!( stitches[ 2 ], Stitch { x : 40, y : 60, instruction : Instruction::Stitch } );
  assert_eq!( stitches[ 3 ], Stitch { x : 40, y : 60, instruction : Instruction::ColorChange } );
  assert_eq!( stitches[ 4 ], Stitch { x : 40, y : 60, instruction : Instruction::Trim } );

  assert_eq!( stitches[ 5 ], Stitch { x : 43, y : 64, instruction : Instruction::Jump } );
  assert_eq!( stitches[ 6 ], Stitch { x : 43, y : 64, instruction : Instruction::Stitch } );

  assert_eq!( stitches[ 7 ], Stitch { x : 43, y : 64, instruction : Instruction::Stop } );
  assert_eq!( stitches[ 8 ], Stitch { x : 43, y : 64, instruction : Instruction::Trim } );

  assert_eq!( stitches[ 9 ], Stitch { x : 63, y : 74, instruction : Instruction::Jump } );
  assert_eq!( stitches[ 10 ], Stitch { x : 63, y : 74, instruction : Instruction::Trim } );

  assert_eq!( stitches[ 11 ], Stitch { x : 64, y : 75, instruction : Instruction::Jump } );
  assert_eq!( stitches[ 12 ], Stitch { x : 64, y : 75, instruction : Instruction::Stitch } );
  assert_eq!( stitches[ 13 ], Stitch { x : 64, y : 75, instruction : Instruction::End } );
}

#[ test ]
fn read_sample_threads_resolve_from_default_palette()
{
  let emb = pec::file_read( "test_files/read_sample.pec" ).unwrap();
  let threads = emb.threads();
  let default_palette = pec::pec_threads();

  assert_eq!( threads[ 0 ], default_palette[ 14 ] );
  assert_eq!( threads[ 1 ], default_palette[ 10 ] );
}

#[ test ]
fn encoding_roundtrip_preserves_stitches_and_threads()
{
  let mut emb = EmbroideryFile::new();
  emb.stitch( 0, 0 );
  emb.stitch( -2, -3 );
  emb.color_change( 0, 0 );
  emb.stitch( 2, 3 );
  emb.trim();
  emb.jump( 40, 30 );
  emb.stitch( 0, 0 );
  emb.stitch( 1, 1 );
  emb.end();

  let threads = pec::pec_threads();
  emb.thread_add( threads[ 1 ].clone() );
  emb.thread_add( threads[ 2 ].clone() );

  let mut memory = vec![ 0_u8; 2048 ];

  {
    let mut writer = Cursor::new( &mut memory );
    pec::write( &mut emb, &mut writer ).unwrap();
  }

  let emb = pec::memory_read( &memory ).unwrap();

  let stitches = emb.stitches();
  assert_eq!( stitches[ 0 ], Stitch { x : 0, y : 0, instruction : Instruction::Stitch } );
  assert_eq!( stitches[ 1 ], Stitch { x : -2, y : -3, instruction : Instruction::Stitch } );
  assert_eq!( stitches[ 2 ], Stitch { x : -2, y : -3, instruction : Instruction::ColorChange } );
  assert_eq!( stitches[ 3 ], Stitch { x : 0, y : 0, instruction : Instruction::Stitch } );
  assert_eq!( stitches[ 4 ], Stitch { x : 0, y : 0, instruction : Instruction::Trim } );
  assert_eq!( stitches[ 5 ], Stitch { x : 40, y : 30, instruction : Instruction::Jump } );
  assert_eq!( stitches[ 6 ], Stitch { x : 40, y : 30, instruction : Instruction::Stitch } );
  assert_eq!( stitches[ 7 ], Stitch { x : 41, y : 31, instruction : Instruction::Stitch } );
  assert_eq!( stitches[ 8 ], Stitch { x : 41, y : 31, instruction : Instruction::End } );

  // Both added threads survive the roundtrip in order. (Pre-BUG-152-fix, the writer
  // unconditionally dropped `emb.threads()[ 0 ]` from the written color table, so only
  // `threads[ 2 ]` came back, landing at position 0 -- a writer defect, not a property of
  // palette entry 1 itself. Deliberately not using `threads[ 0 ]` here: its color (0,0,0)
  // ties with palette index 20's "Black"/Brother entry, and `nearest_color_find`'s `<=`
  // tie-break resolves ties to the *last* matching index -- an unrelated palette-matching
  // property, not something this stitches+threads roundtrip test is meant to exercise.)
  assert_eq!( emb.threads()[ 0 ], threads[ 1 ] );
  assert_eq!( emb.threads()[ 1 ], threads[ 2 ] );
}

// test_kind: bug_reproducer(BUG-151)
/// ## Root Cause
/// `pec_table_process`'s `else` branch (first sighting of a given `color_index`) computed
/// `thread` and inserted it into its dedup `thread_map` but never called
/// `emb.thread_add`/`values.push` -- only the `if let Some(thread)` branch (a repeat
/// sighting of an already-seen `color_index`) did. Every first occurrence of each color
/// was silently dropped instead of recorded, breaking the 1-entry-per-`color_bytes`-byte
/// invariant every downstream consumer relies on.
/// ## Why Not Caught
/// No existing test supplied a non-empty `pes_chart` shorter than the PEC section's color
/// table to `pec::content_read` -- the only way to reach `pec_table_process` at all (a
/// chart that's empty or already `>=` the color count takes a different branch in
/// `pec_colors_map` entirely).
/// ## Fix Applied
/// Added the missing `emb.thread_add`/`values.push` calls to the `else` branch, mirroring
/// what the `if let Some(thread)` branch already did. See `format/pec/reader.rs`.
/// ## Prevention
/// This test writes a design with 2 color-change-delimited stitch runs, reads it back
/// through `pec::content_read` with a 1-entry chart (shorter than the 2-entry color
/// table), and asserts the recovered thread count matches the color table, not the chart.
/// ## Pitfall
/// A dedup cache (`thread_map`, keyed by `color_index`) must only decide *which* value to
/// reuse for a repeat -- it must never gate *whether* a push happens at all; every entry
/// in the source sequence still needs exactly one push, matching the sibling
/// `pec_colors_process`'s unconditional per-byte push.
#[ test ]
fn content_read_with_short_chart_assigns_one_thread_per_color_byte()
{
  let default_palette = pec::pec_threads();

  let mut emb = EmbroideryFile::new();
  emb.stitch( 0, 0 );
  emb.color_change( 0, 0 );
  emb.stitch( 1, 1 );
  emb.end();
  emb.thread_add( default_palette[ 1 ].clone() );
  emb.thread_add( default_palette[ 2 ].clone() );

  let mut memory = vec![ 0_u8; 2048 ];
  {
    let mut writer = Cursor::new( &mut memory );
    pec::write( &mut emb, &mut writer ).unwrap();
  }

  // Skip the 8-byte "#PEC0001" header and feed a chart shorter than the written color
  // table (2 entries), forcing `pec_colors_map` into the `pec_table_process` merge path.
  let mut reader = Cursor::new( &memory );
  reader.set_position( 8 );

  let chart_thread = Thread { description : "chart thread".into(), ..Default::default() };
  let mut result = EmbroideryFile::new();
  pec::content_read( &mut result, &mut reader, std::slice::from_ref( &chart_thread ) ).unwrap();

  let threads = result.threads();
  assert_eq!( threads.len(), 2, "one thread must be recorded per color-table byte, not silently dropped on first sight of a color" );
  assert_eq!( threads[ 0 ], chart_thread, "the first color byte must drain the supplied chart" );
  assert_eq!( threads[ 1 ], default_palette[ 2 ], "chart exhausted -- second color byte falls back to the default palette" );
}

// test_kind: bug_reproducer(BUG-152)
/// ## Root Cause
/// `pec_header_write` sliced `emb.threads()[ 1.. ]` before building the written color
/// table, unconditionally excluding the caller's own first added thread. This was
/// confused with `pec_threads()[ 0 ]`, the *default palette's* dedicated "invalid value"
/// sentinel entry -- an unrelated concept: a documented value inside a fixed 65-entry
/// palette array, not a structural position in the caller's own arbitrary thread list.
/// ## Why Not Caught
/// The one existing roundtrip test happened to add the palette's own sentinel value
/// (`pec_threads()[ 0 ]`, description "Unknown") as its first thread, so the dropped
/// thread's absence was indistinguishable from "the sentinel value doesn't round-trip" --
/// a rationalizing comment recorded that reading instead of the actual defect.
/// ## Fix Applied
/// Changed `&emb.threads()[ 1.. ]` to `emb.threads()` (the full slice) in
/// `pec_header_write`. See `format/pec/writer.rs`.
/// ## Prevention
/// This test adds two ordinary, non-sentinel default-palette threads (indices 1 and 2,
/// deliberately avoiding index 0's sentinel value to keep the evidence unambiguous),
/// round-trips through `pec::write`/`pec::memory_read`, and asserts both survive in order.
/// ## Pitfall
/// A documented sentinel *value* inside a fixed default palette must never be confused
/// with a structural *position* in a caller-supplied, arbitrary-content list -- the two
/// only appeared related here because a prior test happened to use the sentinel value as
/// its own first thread.
#[ test ]
fn encoding_roundtrip_preserves_first_added_thread()
{
  let default_palette = pec::pec_threads();

  let mut emb = EmbroideryFile::new();
  emb.stitch( 0, 0 );
  emb.color_change( 0, 0 );
  emb.stitch( 1, 1 );
  emb.end();
  emb.thread_add( default_palette[ 1 ].clone() );
  emb.thread_add( default_palette[ 2 ].clone() );

  let mut memory = vec![ 0_u8; 2048 ];
  {
    let mut writer = Cursor::new( &mut memory );
    pec::write( &mut emb, &mut writer ).unwrap();
  }

  let emb = pec::memory_read( &memory ).unwrap();

  assert_eq!( emb.threads().len(), 2, "the first added thread must survive the roundtrip, not be silently dropped" );
  assert_eq!( emb.threads()[ 0 ], default_palette[ 1 ] );
  assert_eq!( emb.threads()[ 1 ], default_palette[ 2 ] );
}

/// Builds a minimal PEC content buffer (header-less, matching `content_read`'s own
/// expected layout) with `color_changes = 0` (one color) and a caller-chosen 24-bit
/// `stitch_block_len` placed at its real on-disk offset, so the reader reaches the
/// length-validation code at the exact same point a real file would.
fn build_pec_content_with_stitch_block_len( stitch_block_len : u32 ) -> Vec< u8 >
{
  // 3 ("LA:" skip) + 16 (label) + 0xF + 1 (byte_stride) + 1 (icon_height) + 0xC
  // + 1 (color_changes, set below) + 1 (single color byte, since count_colors = 1)
  // + 0x1D0 (post-color-bytes seek distance when color_changes = 0) = 514 bytes,
  // then the 3-byte `stitch_block_len` field itself.
  let mut buf = vec![ 0u8; 3 + 16 + 0xF + 1 + 1 + 0xC + 1 + 1 + 0x1D0 ];
  buf[ 48 ] = 0; // color_changes = 0 -> count_colors = 1
  buf.extend_from_slice( &stitch_block_len.to_le_bytes()[ ..3 ] );
  buf
}

/// ## Root Cause
/// `content_read` read `stitch_block_len` as an untrusted 24-bit value straight from file
/// data, then computed `stitch_block_len - 5 + reader.stream_position()?` with a raw `-`.
/// For any `stitch_block_len` less than 5, this underflows: it panics in a debug build
/// ("attempt to subtract with overflow") and silently wraps to a value near `u64::MAX` in
/// a release build, corrupting the subsequent `seek( SeekFrom::Start( stitch_block_end ) )`.
/// This code path is reachable both directly (`pec::file_read`/`memory_read`/`read`) and
/// indirectly via `pes::file_read`/`memory_read`/`read`, since PES files embed a PEC
/// content block parsed by this same `content_read` function.
///
/// ## Why Not Caught
/// No existing test constructed a PEC buffer with a corrupted/malicious `stitch_block_len`
/// -- all existing tests use either the reference sample file or a freshly-written buffer
/// from this crate's own writer, both of which always produce a valid (>= 5) length.
///
/// ## Fix Applied
/// Replaced the raw `stitch_block_len - 5` with `stitch_block_len.checked_sub( 5 )
/// .ok_or_else( .. )?`, returning `EmbroideryError::DecodingError` for any length under 5
/// instead of panicking or silently wrapping. See `format/pec/reader.rs`.
///
/// ## Prevention
/// This test builds a minimal PEC content buffer with `stitch_block_len = 0` (the
/// simplest under-5 value) and asserts `content_read` returns a `DecodingError` instead of
/// panicking or succeeding with a corrupted read position.
///
/// ## Pitfall
/// Arithmetic on any length/offset value read from untrusted file data must use
/// `checked_*` and return a decode error -- a raw operator either panics (debug) or wraps
/// to a wildly wrong value that gets used as a real seek position (release), neither of
/// which is a safe way to handle malformed input.
// BUG-314 task/bug/314_pec_stitch_block_len_underflow.md -- reproducer for the untrusted
// `stitch_block_len` underflowing when less than 5, reachable via both `pec::*` and
// `pes::*` public entry points.
// test_kind: bug_reproducer(BUG-314)
#[ test ]
fn content_read_rejects_stitch_block_len_below_5_instead_of_underflowing()
{
  let buf = build_pec_content_with_stitch_block_len( 0 );
  let mut cursor = Cursor::new( buf );
  let mut emb = EmbroideryFile::new();

  let result = pec::content_read( &mut emb, &mut cursor, &[] );

  assert!(
    matches!( result, Err( EmbroideryError::DecodingError( _ ) ) ),
    "a stitch block length under 5 must return a DecodingError, not panic or succeed \
    with a corrupted read position; got {result:?}"
  );
}

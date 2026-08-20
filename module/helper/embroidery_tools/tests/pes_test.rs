//! PES format tests — writer output pinned byte-for-byte against reference fixtures
//! in `test_files/`, and a write→read roundtrip preserving metadata and threads.
//! Public API only.

use std::io::Cursor;
use embroidery_tools::embroidery_file::EmbroideryFile;
use embroidery_tools::error::EmbroideryError;
use embroidery_tools::format::{ pec, pes };
use embroidery_tools::thread::{ Color, Thread };

/// Builds the stitch program the reference files in `test_files/` were generated from.
fn fixture_program() -> EmbroideryFile
{
  let mut emb = EmbroideryFile::new();
  emb.stitch( 0, 0 );
  emb.stitch( -40, -30 );
  emb.color_change( 0, 0 );
  emb.trim();
  emb.jump( 2, 3 );
  emb.stitch( 0, 0 );
  emb.trim();
  emb.jump( 90, -100 );
  emb.stitch( 0, 0 );
  emb.stitch( 1, 1 );
  emb.end();

  let threads = pec::pec_threads();
  emb.thread_add( threads[ 1 ].clone() );
  emb.thread_add( threads[ 2 ].clone() );

  emb
}

#[ test ]
fn write_v1_matches_reference_fixture()
{
  let sample = std::fs::read( "test_files/pes_test_v1.pes" ).unwrap();
  let mut emb = fixture_program();

  let mut memory = vec![ 0_u8; 4096 ];
  {
    let mut writer = Cursor::new( &mut memory );
    pes::write( &mut emb, &mut writer, pes::PESVersion::V1 ).unwrap();
  }

  // 192 is the index where the PES section ends and the PEC section starts
  // specifically in this file.
  assert_eq!( &memory[ ..192 ], &sample[ ..192 ] );
}

#[ test ]
fn write_v6_matches_reference_fixture()
{
  let sample = std::fs::read( "test_files/pes_test_v6.pes" ).unwrap();
  let mut emb = fixture_program();

  let mut memory = vec![ 0_u8; 4096 ];
  {
    let mut writer = Cursor::new( &mut memory );
    pes::write( &mut emb, &mut writer, pes::PESVersion::V6 ).unwrap();
  }

  // 361 is the index where the PES section ends and the PEC section starts
  // specifically in this file.
  assert_eq!( &memory[ ..361 ], &sample[ ..361 ] );
}

#[ test ]
fn v6_roundtrip_preserves_metadata_and_threads()
{
  let mut emb = EmbroideryFile::new();
  emb.stitch( 0, 0 );
  emb.end();
  let metadata = emb.metadata_get_mut();
  metadata.text_insert( "category", "Fantasy".into() );
  metadata.text_insert( "author", "George R.R. Martin".into() );
  metadata.text_insert( "keywords", "Dragons, mediavel, story, adventure".into() );
  metadata.text_insert( "comments", "When \"The Winds of Winter\"?".into() );

  let color = Color { r : 123, g : 234, b : 125 };
  let thread = Thread
  {
    color,
    description : "A very good thread".into(),
    catalog_number : "197".into(),
    brand : "No brand".into(),
    chart : "No chart".into(),
    ..Default::default()
  };
  emb.thread_add( thread );

  let mut memory = vec![ 0_u8; 2048 ];
  {
    let mut writer = Cursor::new( &mut memory );
    pes::write( &mut emb, &mut writer, pes::PESVersion::V6 ).unwrap();
  }

  let mut reader = Cursor::new( &mut memory );
  let emb = pes::read( &mut reader ).unwrap();
  let metadata = emb.metadata_get();

  assert_eq!( metadata.text_get( "category" ).unwrap(), "Fantasy" );
  assert_eq!( metadata.text_get( "author" ).unwrap(), "George R.R. Martin" );
  assert_eq!( metadata.text_get( "keywords" ).unwrap(), "Dragons, mediavel, story, adventure" );
  assert_eq!( metadata.text_get( "comments" ).unwrap(), "When \"The Winds of Winter\"?" );

  let thread = &emb.threads()[ 0 ];
  assert_eq!( thread.description, "A very good thread" );
  assert_eq!( thread.catalog_number, "197" );
  assert_eq!( thread.brand, "No brand" );
  assert_eq!( thread.chart, "No chart" );
}

// test_kind: bug_reproducer(BUG-234)
/// ## Root Cause
/// `pes_addendum_write`'s fixed 128-byte color-index field computed its padding via
/// `128_usize.wrapping_sub( count )`, but the only existing guard on `count`
/// (`pec_header_write`'s own "too many color changes" check) allows up to 255 -- for any
/// `count` in `129..=255`, `wrapping_sub` silently underflowed to a value near
/// `usize::MAX`, which then became a `Vec<u8>` allocation size far past `isize::MAX`,
/// panicking with "capacity overflow" instead of returning a catchable error.
/// ## Why Not Caught
/// No existing test wrote a design with more than a handful of threads -- the largest,
/// `write_v6_matches_reference_fixture`, uses only 2.
/// ## Fix Applied
/// Added an explicit `count > 128` guard in `pes_addendum_write` returning
/// `EmbroideryError::CompatibilityError`, mirroring `pec_header_write`'s own "too many
/// color changes" check. See `format/pes/writer.rs`.
/// ## Prevention
/// This test writes a design with 129 threads (one past the addendum's 128-byte capacity)
/// to PES v6 and asserts a `CompatibilityError` comes back instead of a panic.
/// ## Pitfall
/// Every other "value exceeds this format's capacity" case in this file reports through
/// `try_from`/an explicit bounds check and a real `EmbroideryError` -- `wrapping_sub` fed
/// straight into an allocation size was the one place that convention wasn't followed.
#[ test ]
fn version6_write_with_more_than_128_threads_errors_instead_of_panicking()
{
  let mut emb = EmbroideryFile::new();
  emb.stitch( 0, 0 );
  emb.end();

  let default_palette = pec::pec_threads();
  for i in 0..129
  {
    emb.thread_add( default_palette[ 1 + ( i % ( default_palette.len() - 1 ) ) ].clone() );
  }

  let mut memory = vec![ 0_u8; 4096 ];
  let mut writer = Cursor::new( &mut memory );
  let result = pes::write( &mut emb, &mut writer, pes::PESVersion::V6 );

  assert!
  (
    matches!( result, Err( EmbroideryError::CompatibilityError( _ ) ) ),
    "writing a design with 129 threads must return CompatibilityError, not panic or succeed: {result:?}"
  );
}

// test_kind: bug_reproducer(BUG-235)
/// ## Root Cause
/// `as_segment_blocks` unconditionally called `thread::nearest_color_find( &color, &chart
/// ).unwrap()` (twice: once before its main loop, once on every `ColorChange` instruction),
/// where `chart` is built from `emb.threads()`. A design that never had a thread added and
/// has no `Stitch`/`SewTo`/`NeedleAt` instruction (so `color_count_fix` never backfills one)
/// reaches this call with an empty `chart` -- `nearest_color_find` returns `None` by
/// contract for an empty palette, and `.unwrap()` panicked instead of degrading gracefully.
/// ## Why Not Caught
/// Every existing PES v6 test adds at least one thread via `thread_add` before writing --
/// none exercised a design with zero threads.
/// ## Fix Applied
/// Changed both `.unwrap()` calls in `as_segment_blocks` to `.unwrap_or( 0 )`, matching this
/// section's own write-only/informational nature (this codebase's PES reader never reads it
/// back) and the same "substitute something reasonable instead of erroring" convention
/// `thread_or_filler_get` already uses. See `format/pes/writer.rs`.
/// ## Prevention
/// This test writes a design with a single `end()` instruction and zero added threads to
/// PES v6 and asserts the call succeeds instead of panicking.
/// ## Pitfall
/// `emb.stitches().is_empty()` (checked by `pes_block_write` to skip the whole CEmbOne/
/// CSewSeg block) is not the same condition as "zero threads" -- a jump-only or otherwise
/// stitch-free-but-non-empty instruction sequence still reaches `as_segment_blocks` with
/// however many threads the design happens to have, which can be zero.
#[ test ]
fn version6_write_with_zero_threads_does_not_panic()
{
  let mut emb = EmbroideryFile::new();
  emb.end();
  assert!( emb.threads().is_empty(), "test setup: design must have zero threads to reach the empty-chart path" );

  let mut memory = vec![ 0_u8; 4096 ];
  let mut writer = Cursor::new( &mut memory );
  let result = pes::write( &mut emb, &mut writer, pes::PESVersion::V6 );

  assert!( result.is_ok(), "writing a threadless design to PES v6 must not panic: {result:?}" );
}

/// Decodes the ordered list of `( flag, points )` segment blocks from the `CSewSeg` section of
/// PES bytes written by `pes::write`. `segment_count` is the number of segment blocks the
/// caller's own instruction sequence is known to produce (this decoder does not infer it, so
/// callers must derive it from `EmbroideryFile::as_command_blocks`'s consecutive-same-instruction
/// grouping -- see the reproducer below).
///
/// Mirrors exactly as much of `pes_embsewseg_segments_write`'s binary layout
/// (`format/pes/writer.rs`) as is needed: `[ flag : u16, color_code : u16, point_count : u16,
/// ( x : i16, y : i16 ) * point_count ]` per segment, with a `0x8003` marker written between
/// (not around) consecutive segments.
fn csewseg_segments_decode( pes_bytes : &[ u8 ], segment_count : usize ) -> Vec< ( u16, Vec< ( i16, i16 ) > ) >
{
  // "CSewSeg" is `pes_string16_write`'s 7 ASCII bytes following its own u16-LE length prefix --
  // distinctive enough in this small, fully-controlled fixture to locate unambiguously.
  let marker = b"CSewSeg";
  let marker_pos = pes_bytes.windows( marker.len() )
  .position( | w | w == marker )
  .expect( "\"CSewSeg\" marker not found in written PES bytes -- has the CEmbOne/CSewSeg block layout changed?" );
  let mut pos = marker_pos + marker.len();

  let mut segments = vec![];
  for i in 0..segment_count
  {
    // Between (not around) consecutive segments, the writer emits a `0x8003` "next segment"
    // marker -- consume it before every segment after the first.
    if i > 0
    {
      let marker_word = u16::from_le_bytes( [ pes_bytes[ pos ], pes_bytes[ pos + 1 ] ] );
      assert_eq!( marker_word, 0x8003, "expected the 0x8003 inter-segment marker before segment {i}" );
      pos += 2;
    }

    let flag = u16::from_le_bytes( [ pes_bytes[ pos ], pes_bytes[ pos + 1 ] ] );
    pos += 2;
    let _color_code = u16::from_le_bytes( [ pes_bytes[ pos ], pes_bytes[ pos + 1 ] ] );
    pos += 2;
    let point_count = u16::from_le_bytes( [ pes_bytes[ pos ], pes_bytes[ pos + 1 ] ] );
    pos += 2;

    let mut points = vec![];
    for _ in 0..point_count
    {
      let x = i16::from_le_bytes( [ pes_bytes[ pos ], pes_bytes[ pos + 1 ] ] );
      let y = i16::from_le_bytes( [ pes_bytes[ pos + 2 ], pes_bytes[ pos + 3 ] ] );
      points.push( ( x, y ) );
      pos += 4;
    }

    segments.push( ( flag, points ) );
  }

  segments
}

// test_kind: bug_reproducer(BUG-341)
/// ## Root Cause
/// `as_segment_blocks`'s `Instruction::Jump` arm reads `stitched_x`/`stitched_y` to compute a
/// jump segment's start point, but -- unlike the `Instruction::Stitch` arm, which writes both
/// back after every stitch -- never writes them back itself. When a `ColorChange` (or any other
/// non-`Stitch` instruction, all of which fall through a catch-all `_ => continue` that also
/// never touches the tracker) separates two `Jump` command-blocks with no intervening `Stitch`,
/// the second jump's recorded start point is whatever the tracker held before the FIRST jump,
/// not where the first jump actually ended.
/// ## Why Not Caught
/// No existing PES writer test exercised two `Jump` command-blocks separated only by a
/// non-moving instruction (`ColorChange`, `Trim`, etc.) with no `Stitch` between them -- every
/// existing fixture's jumps are either isolated by stitches on both sides or immediately
/// followed by a stitch, which happens to refresh the tracker before the next jump reads it.
/// ## Fix Applied
/// Added `stitched_x = last_instruction.x; stitched_y = last_instruction.y;` to the `Jump` arm,
/// mirroring what the `Stitch` arm already does, so a jump's endpoint becomes the tracked
/// position for whatever segment reads it next. See `format/pes/writer.rs`.
/// ## Prevention
/// This test writes two jumps separated by a no-op `color_change` and decodes the actual
/// `CSewSeg` segment bytes `pes::write` produced, asserting the second jump segment's start
/// point equals the first jump segment's end point (the only physically correct value, since no
/// movement occurs between them) rather than the first jump segment's own start point (the stale
/// value the bug produces).
/// ## Pitfall
/// `as_segment_blocks`'s CSewSeg output is write-only/informational -- this crate's own
/// `pes::read` never reads it back (confirmed by BUG-235's precedent) -- so this bug is invisible
/// to any roundtrip (`write` then `read`) test; it can only be caught by inspecting the raw bytes
/// `pes::write` actually produced.
#[ test ]
fn second_jump_after_colorchange_starts_where_first_jump_ended()
{
  let mut emb = EmbroideryFile::new();
  emb.stitch( 0, 0 );
  emb.stitch( 10, 10 );
  emb.jump( 5, 5 );
  emb.color_change( 0, 0 );
  emb.jump( 5, 5 );
  emb.stitch( 0, 0 );
  emb.stitch( 1, 1 );
  emb.end();

  let mut memory = vec![ 0_u8; 4096 ];
  {
    let mut writer = Cursor::new( &mut memory );
    pes::write( &mut emb, &mut writer, pes::PESVersion::V6 ).unwrap();
  }

  // Expected segment layout for this exact instruction sequence: [Stitch, Jump, Jump, Stitch] --
  // `ColorChange` never becomes its own segment (see `as_segment_blocks`'s `continue` arm), and
  // two consecutive `stitch()`/two consecutive `jump()` calls each stay within one command block
  // (`EmbroideryFile::as_command_blocks` only splits where the instruction type changes).
  let segments = csewseg_segments_decode( &memory, 4 );
  let flags : Vec< u16 > = segments.iter().map( | ( flag, _ ) | *flag ).collect();
  assert_eq!( flags, vec![ 0, 1, 1, 0 ], "test setup: expected [stitch, jump, jump, stitch] segments, got flags {flags:?}" );

  let jump_1_end = *segments[ 1 ].1.last().unwrap();
  let jump_2_start = segments[ 2 ].1[ 0 ];

  assert_eq!
  (
    jump_2_start, jump_1_end,
    "second jump segment must start where the first jump ended ({jump_1_end:?}), not carry over a stale pre-first-jump needle position (found {jump_2_start:?})"
  );
}

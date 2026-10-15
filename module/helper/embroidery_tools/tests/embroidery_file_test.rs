//! Behavior tests for `EmbroideryFile` — stitch accumulation, bounds, and
//! command-block splitting, exercised through the crate's public API only.

use embroidery_tools::embroidery_file::EmbroideryFile;
use embroidery_tools::stitch_instruction::{ Instruction, Stitch };

#[ test ]
fn add_stitch_relative_accumulates_positions()
{
  let mut emb = EmbroideryFile::new();
  emb.stitch_add_relative( Stitch { x : 10, y : 20, instruction : Instruction::Stitch } );
  emb.stitch_add_relative( Stitch { x : 30, y : 40, instruction : Instruction::Stitch } );

  let stitches = emb.stitches();

  assert_eq!( stitches[ 0 ], Stitch { x : 10, y : 20, instruction : Instruction::Stitch } );
  assert_eq!( stitches[ 1 ], Stitch { x : 40, y : 60, instruction : Instruction::Stitch } );
}

#[ test ]
fn add_stitch_absolute_keeps_positions()
{
  let mut emb = EmbroideryFile::new();
  emb.stitch_add_absolute( Stitch { x : 10, y : 20, instruction : Instruction::Stitch } );
  emb.stitch_add_absolute( Stitch { x : 30, y : 40, instruction : Instruction::Stitch } );

  let stitches = emb.stitches();

  assert_eq!( stitches[ 0 ], Stitch { x : 10, y : 20, instruction : Instruction::Stitch } );
  assert_eq!( stitches[ 1 ], Stitch { x : 30, y : 40, instruction : Instruction::Stitch } );
}

#[ test ]
fn bounds_returns_min_and_max_stitch_coordinates()
{
  let mut emb = EmbroideryFile::new();
  emb.stitch_add_absolute( Stitch { x : -5, y : 40, instruction : Instruction::Stitch } );
  emb.stitch_add_absolute( Stitch { x : 30, y : -20, instruction : Instruction::Stitch } );
  emb.stitch_add_absolute( Stitch { x : 10, y : 15, instruction : Instruction::Jump } );

  // ( min_x, min_y, max_x, max_y )
  assert_eq!( emb.bounds(), Some( ( -5, -20, 30, 40 ) ) );
}

// test_kind: bug_reproducer(BUG-497)
/// ## Root Cause
/// `bounds()` seeded `min_x`/`min_y` at `i32::MAX` and `max_x`/`max_y` at
/// `i32::MIN`, then returned the tuple unchanged whenever `self.stitches()`
/// was empty -- an inverted sentinel (`min > max`) indistinguishable from a
/// real bounds value to any caller, which overflows/panics if that caller
/// computes a width/height via `max_x - min_x` (`i32::MIN - i32::MAX`
/// underflows `i32`).
/// ## Why Not Caught
/// The only pre-existing test (`bounds_returns_min_and_max_stitch_coordinates`
/// above) always added stitches before calling `bounds()`, so the
/// zero-stitch path was never exercised.
/// ## Fix Applied
/// `bounds()` (`src/embroidery_file.rs`) now returns
/// `Option< ( i32, i32, i32, i32 ) >` -- `None` when `self.stitches()` is
/// empty, seeding the min/max reduction from the first real stitch instead
/// of a sentinel constant. Call sites in `format::pec::writer::content_write`
/// and `format::pes::writer::{version1_write,version6_write}` updated to
/// handle the `None` case explicitly.
/// ## Prevention
/// This test locks in `None` for a freshly-constructed, stitch-free file.
/// ## Pitfall
/// A min/max reduction seeded from sentinel constants is only safe when the
/// collection being reduced is provably non-empty -- for a possibly-empty
/// collection, the seed itself is a fabricated value with no real-world
/// meaning, and returning it unchanged silently manufactures a fake result
/// instead of surfacing the absence of data.
#[ test ]
fn bounds_returns_none_for_empty_file()
{
  let emb = EmbroideryFile::new();
  assert_eq!( emb.bounds(), None );
}

#[ test ]
fn as_command_blocks_splits_at_instruction_changes()
{
  let mut emb = EmbroideryFile::new();
  emb.stitch_add_absolute( Stitch { x : 0, y : 0, instruction : Instruction::Stitch } );
  emb.stitch_add_absolute( Stitch { x : 1, y : 1, instruction : Instruction::Stitch } );
  emb.stitch_add_absolute( Stitch { x : 2, y : 2, instruction : Instruction::Jump } );
  emb.stitch_add_absolute( Stitch { x : 3, y : 3, instruction : Instruction::Stitch } );

  let blocks = emb.as_command_blocks();

  // Splits sit where the instruction changes: [ Stitch, Stitch ], [ Jump ], [ Stitch ].
  assert_eq!( blocks.len(), 3 );
  assert_eq!
  (
    blocks[ 0 ],
    vec!
    [
      Stitch { x : 0, y : 0, instruction : Instruction::Stitch },
      Stitch { x : 1, y : 1, instruction : Instruction::Stitch },
    ]
  );
  assert_eq!( blocks[ 1 ], vec![ Stitch { x : 2, y : 2, instruction : Instruction::Jump } ] );
  assert_eq!( blocks[ 2 ], vec![ Stitch { x : 3, y : 3, instruction : Instruction::Stitch } ] );
}

// test_kind: bug_reproducer(BUG-150)
/// ## Root Cause
/// `duplicate_color_interpolate_as_stop`'s guard compared `self.threads().get( thread_index )`
/// against `self.threads().get( thread_index - 1 )` with no bounds check. When there are more
/// color-change-delimited stitch runs than recorded threads, both `.get()` calls return `None`,
/// and `None == None` is `true` in Rust, so the guard was satisfied and
/// `self.threads.remove( thread_index )` ran on an out-of-range index.
/// ## Why Not Caught
/// No existing test called `duplicate_color_interpolate_as_stop` at all, and its sibling
/// `stop_interpolate_as_duplicate_color` already has the correct `thread_index < len` guard, so
/// there was no precedent failure to compare against.
/// ## Fix Applied
/// Added a `thread_index < self.threads().len()` guard before the `.get()` comparison,
/// mirroring `stop_interpolate_as_duplicate_color`'s existing bounds-check pattern. See
/// `embroidery_file.rs`.
/// ## Prevention
/// This test constructs a file with more color-change-delimited stitch runs than threads (zero
/// threads at all), which is exactly the shape `pec::content_read` can produce from a malformed
/// or unusual PEC/PES file, since it calls this function automatically on every read.
/// ## Pitfall
/// `None == None` reads as "these two threads are equal" instead of "neither index is valid" --
/// any `Option`-returning `.get()` comparison used as an equality check must first confirm at
/// least one side is genuinely in-bounds, or two absences will silently compare as a match.
#[ test ]
fn duplicate_color_interpolate_as_stop_does_not_panic_with_fewer_threads_than_color_changes()
{
  let mut emb = EmbroideryFile::new();
  emb.color_change( 0, 0 );
  emb.stitch( 1, 1 );
  emb.color_change( 0, 0 );
  emb.stitch( 1, 1 );

  emb.duplicate_color_interpolate_as_stop();
}

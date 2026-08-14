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
  assert_eq!( emb.bounds(), ( -5, -20, 30, 40 ) );
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

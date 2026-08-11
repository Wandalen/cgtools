//! PEC format tests — decoding the reference sample in `test_files/` and a
//! write→read roundtrip preserving stitches and thread selection. Public API only.

use std::io::Cursor;
use embroidery_tools::embroidery_file::EmbroideryFile;
use embroidery_tools::format::pec;
use embroidery_tools::stitch_instruction::{ Instruction, Stitch };

#[ test ]
fn read_sample_stitches_match_reference_decoder()
{
  let emb = pec::read_file( "test_files/read_sample.pec" ).unwrap();
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
  let emb = pec::read_file( "test_files/read_sample.pec" ).unwrap();
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
  emb.add_thread( threads[ 0 ].clone() );
  emb.add_thread( threads[ 2 ].clone() );

  let mut memory = vec![ 0_u8; 2048 ];

  {
    let mut writer = Cursor::new( &mut memory );
    pec::write( &mut emb, &mut writer ).unwrap();
  }

  let emb = pec::read_memory( &memory ).unwrap();

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

  // Thread 0 is the palette's "invalid value" marker; the roundtrip resolves the
  // first real thread, palette entry 2.
  assert_eq!( emb.threads()[ 0 ], threads[ 2 ] );
}

//!
//! # Represents embroidery file
//!

mod private
{
  //!
  //! # Represents embroidery file
  //!
  use crate::*;
  use thread::*;
  use metadata::Metadata;
  use stitch_instruction::{ Instruction, Stitch };

  /// Represents embroidery file. Stores embroidery instructions, threads and metadata
  #[ derive( Debug, Clone ) ]
  pub struct EmbroideryFile
  {
    metadata : Metadata,
    threads : Vec< Thread >,
    // stores stitches with absolute coordinates
    stitches : Vec< Stitch >,
    // coordinates of last stitch instruction
    prev_x : i32,
    prev_y : i32,
  }

  impl EmbroideryFile
  {
    /// Creates new instance of `EmbroideryFile`
    pub fn new() -> Self
    {
      Self
      {
        stitches : vec![],
        threads : vec![],
        prev_x : 0,
        prev_y : 0,
        metadata : Metadata::new(),
      }
    }

    /// Returns stitches with absolute coordinates
    pub fn stitches( &self ) -> &[ Stitch ]
    {
      &self.stitches
    }

    /// Threads of embroidery file
    pub fn threads( &self ) -> &[ Thread ]
    {
      &self.threads
    }

    /// Gets mutable metadata
    pub fn get_mut_metadata( &mut self ) -> &mut Metadata
    {
      &mut self.metadata
    }

    /// Gets metadata
    pub fn get_metadata( &self ) -> &Metadata
    {
      &self.metadata
    }

    /// Adds stitch instruction with relative coordinates
    pub fn stitch( &mut self, dx : i32, dy : i32 )
    {
      self.add_stitch_relative( Stitch { x : dx, y : dy, instruction : Instruction::Stitch } );
    }

    /// Adds jump instruction with relative coordinates
    pub fn jump( &mut self, dx : i32, dy : i32 )
    {
      self.add_stitch_relative( Stitch { x : dx, y : dy, instruction : Instruction::Jump } );
    }

    /// Adds color change instruction with relative coordinates
    pub fn color_change( &mut self, dx : i32, dy : i32 )
    {
      self.add_stitch_relative( Stitch { x : dx, y : dy, instruction : Instruction::ColorChange } );
    }

    /// Adds trim instruction with relative coordinates at [0; 0]
    pub fn trim( &mut self )
    {
      self.add_stitch_relative( Stitch { x : 0, y : 0, instruction : Instruction::Trim } );
    }

    /// Adds end instruction with relative coordinates at [0; 0]
    pub fn end( &mut self )
    {
      self.add_stitch_relative( Stitch { x : 0, y : 0, instruction : Instruction::End } );
    }

    /// Adds stitch instruction, assuming that coodinates are relative
    pub fn add_stitch_relative( &mut self, mut stitch : Stitch )
    {
      // Convert to absolute
      stitch.x += self.prev_x;
      stitch.y += self.prev_y;
      self.add_stitch_absolute( stitch );
    }

    /// Adds stitch instruction, assuming that coodinates are absolute
    pub fn add_stitch_absolute( &mut self, stitch : Stitch )
    {
      self.prev_x = stitch.x;
      self.prev_y = stitch.y;
      self.stitches.push( stitch );
    }

    /// Adds thread to palette
    pub fn add_thread( &mut self, thread : Thread )
    {
      self.threads.push( thread );
    }

    /// Returns thread in `index` or a random thread.
    /// Currently PEC pallete is used for random thread sampling
    pub fn get_thread_or_filler( &self, index : usize ) -> Thread
    {
      self.threads.get( index ).unwrap_or( &thread::get_random_thread() ).clone()
    }

    /// This function replaces duplicate color changes with `Stop` instruciton.
    /// Should be used when reading specific formats where stop instruction is encoded
    /// with duplicate color change
    pub fn interpolate_duplicate_color_as_stop( &mut self )
    {
      let mut thread_index = 0;
      let mut init_color = true;
      let mut last_change = None;

      for i in 0..self.stitches.len()
      {
        let instruction = self.stitches[ i ].instruction;

        if ( instruction == Instruction::Stitch
        || instruction == Instruction::SewTo
        || instruction == Instruction::NeedleAt )
        && init_color
        {
          if thread_index != 0
          && self.threads().get( thread_index ) == self.threads().get( thread_index - 1 )
          && last_change.is_some()
          {
            _ = self.threads.remove( thread_index );
            let last_change : usize = last_change.unwrap();
            self.stitches[ last_change ].instruction = Instruction::Stop;
          }
          else
          {
            thread_index += 1;
          }

          init_color = false;
        }
        else if instruction == Instruction::ColorChange
        || instruction == Instruction::ColorBreak
        || instruction == Instruction::NeedleSet
        {
          init_color = true;
          last_change = Some( i );
        }
      }
    }

    /// This function brings embroidery file
    /// into specific form where Stop instruction is replaced
    /// with duplicated color change. Should be used when writing
    /// specific formats where Stop instruction should be encoded as
    /// duplicate color change
    pub fn interpolate_stop_as_duplicate_color( &mut self )
    {
      let mut thread_index = 0;
      for i in 0..self.stitches.len()
      {
        let stitch = self.stitches[ i ];
        let instruction = stitch.instruction;
        if instruction == Instruction::Stitch
        || instruction == Instruction::SewTo
        || instruction == Instruction::NeedleAt
        {
          continue;
        }
        else if instruction == Instruction::ColorChange
        || instruction == Instruction::ColorBreak
        || instruction == Instruction::NeedleSet
        {
          thread_index += 1;
        }
        else if instruction == Instruction::Stop
        {
          if thread_index < self.threads.len()
          {
            let insert = self.threads[ thread_index ].clone();
            self.threads.insert( thread_index, insert );
            self.stitches[ i ].instruction = Instruction::ColorChange;
            thread_index += 1;
          }
          else
          {
            // No colors to duplicate
            return;
          }
        }
      }
    }

    /// This function ensures that there is a enough threads
    /// for every color change. If it is not then it adds some random threads
    pub fn fix_color_count( &mut self )
    {
      let mut thread_index = 0;
      let mut init_color = true;

      for stitch in &self.stitches
      {
        let instruction = stitch.instruction;
        if ( instruction == Instruction::Stitch
        || instruction == Instruction::SewTo
        || instruction == Instruction::NeedleAt )
        && init_color
        {
          thread_index += 1;
          init_color = false;
        }
        else if instruction == Instruction::ColorChange
        || instruction == Instruction::ColorBreak
        || instruction == Instruction::NeedleSet
        {
          init_color = true;
        }
      }

      while self.threads.len() < thread_index
      {
        self.add_thread( self.get_thread_or_filler( self.threads().len() ) );
      }
    }

    /// Minimum and maximum coordinates of stitches.
    /// # Returns
    /// Pairs of min X min Y and max X max Y
    pub fn bounds( &self ) -> ( i32, i32, i32, i32 )
    {
      let mut max_x = i32::MIN;
      let mut min_x = i32::MAX;
      let mut max_y = i32::MIN;
      let mut min_y = i32::MAX;

      for stitch in self.stitches()
      {
        max_x = max_x.max( stitch.x );
        min_x = min_x.min( stitch.x );
        max_y = max_y.max( stitch.y );
        min_y = min_y.min( stitch.y );
      }

      ( min_x, min_y, max_x, max_y )
    }

    /// Returns blocks of stitches splitted at positions where
    /// instructions doesn't repeat. Currently used for PES encoding
    pub fn as_command_blocks( &self ) -> Vec< Vec< Stitch > >
    {
      let mut ret = vec![];
      let mut last_pos = 0;
      let mut last_instruction = Instruction::NoInstruction;
      for ( i, stitch ) in self.stitches().iter().enumerate()
      {
        let instruction = stitch.instruction;
        if instruction == last_instruction || last_instruction == Instruction::NoInstruction
        {
          last_instruction = instruction;
          continue;
        }
        last_instruction = instruction;
        ret.push( self.stitches()[ last_pos..i ].to_owned() );
        last_pos = i;
      }
      ret.push( self.stitches()[ last_pos.. ].to_owned() );
      ret
    }
  }

  impl Default for EmbroideryFile
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  #[ cfg( test ) ]
  mod test
  {
    use super::*;
    use stitch_instruction::*;

    #[ test ]
    fn test_add_stitch_relative()
    {
      let mut emb = EmbroideryFile::new();
      emb.add_stitch_relative( Stitch { x : 10, y : 20, instruction: Instruction::Stitch } );
      emb.add_stitch_relative( Stitch { x : 30, y : 40, instruction: Instruction::Stitch } );

      let stitches = emb.stitches();

      assert_eq!( stitches[ 0 ], Stitch { x : 10, y : 20, instruction: Instruction::Stitch } );
      assert_eq!( stitches[ 1 ], Stitch { x : 40, y : 60, instruction: Instruction::Stitch } );
    }

    #[ test ]
    fn test_add_stitch_absolute()
    {
      let mut emb = EmbroideryFile::new();
      emb.add_stitch_absolute( Stitch { x : 10, y : 20, instruction: Instruction::Stitch } );
      emb.add_stitch_absolute( Stitch { x : 30, y : 40, instruction: Instruction::Stitch } );

      let stitches = emb.stitches();

      assert_eq!( stitches[ 0 ], Stitch { x : 10, y : 20, instruction: Instruction::Stitch } );
      assert_eq!( stitches[ 1 ], Stitch { x : 30, y : 40, instruction: Instruction::Stitch } );
    }
  }
}

crate::mod_interface!
{
  own use EmbroideryFile;
}

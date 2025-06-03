//! 
//! # Embroidery instruction representation
//! 

mod private
{
  /// General low-level embroidery instructions. 
  /// Doesn't map 1:1 to actual binary instructions for all formats.
  /// Format encoders and decoders are responsible 
  /// of mapping actual binary instructions to these more general ones
  #[ non_exhaustive ]
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub enum Instruction
  {
    /// Instruction absence
    NoInstruction,
    /// A puncture
    Stitch,
    /// Needle translation
    Jump,
    /// Thread trim
    Trim,
    /// Indicates change of a thread
    ColorChange,
    /// Indicates stop of an embroidery machine
    Stop,
    /// Indicates end of instructions
    End,
    /// This instruction is for future compatibility
    SewTo,
    /// This instruction is for future compatibility
    NeedleAt,
    /// This instruction is for future compatibility
    ColorBreak,
    /// This instruction is for future compatibility
    NeedleSet,
  }

  /// Stores instruction and coordinates of its appliance
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  pub struct Stitch
  {
    /// X coordinate of instruction
    pub x : i32,
    /// Y coordinate of instruction
    pub y : i32,
    /// Instruction, which is applied at the coordinates
    pub instruction : Instruction
  }
}

crate::mod_interface!
{
  own use Instruction;
  own use Stitch;
}

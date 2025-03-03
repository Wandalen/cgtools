//!
//! # Embroidery instruction representation
//!

mod private {
    /// General low-level embroidery instructions.
    /// Doesn't map 1:1 to actual binary instructions for all formats.
    /// Format encoders and decoders are responsible
    /// of mapping actual binary instructions to these more general ones
    #[non_exhaustive]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(i32)]
    pub enum Instruction {
        /// Instruction absence
        NoInstruction = -1,
        /// A puncture
        Stitch = 0,
        /// Needle translation
        Jump = 1,
        /// Thread trim
        Trim = 2,
        /// Indicates stop of an embroidery machine
        Stop = 3,
        /// Indicates end of instructions
        End = 4,
        /// Indicates change of a thread
        ColorChange = 5,
        NeedleSet = 9,
        SequinMode = 6,
        SequinEject = 7,
        Slow = 0xB,
        Fast = 0xC,

        SetChangeSequence = 0x10,

        SewTo = 0xB0,
        NeedleAt = 0xB1,

        StitchBreak = 0xE0,

        SequenceBreak = 0xE1,
        ColorBreak = 0xE2,

        TieOn = 0xE4,
        TieOff = 0xE5,
        FrameEject = 0xE9,

        MatrixTranslate = 0xC0,
        MatrixScaleOrigin = 0xC1,
        MatrixRotateOrigin = 0xC2,
        MatrixScale = 0xC4,
        MatrixRotate = 0xC5,
        MatrixReset = 0xC3,

        OptionMaxStitchLength = 0xD5,
        OptionMaxJumpLength = 0xD6,
        OptionExplicitTrim = 0xD7,
        OptionImplicitTrim = 0xD8,

        ContingencyTieOnNone = 0xD3,
        ContingencyTieOnThreeSmall = 0xD1,

        ContingencyTieOffNone = 0xD4,
        ContingencyTieOffThreeSmall = 0xD2,

        ContingencyLongStitchNone = 0xF0,
        ContingencyLongStitchJumpNeedle = 0xF1,
        ContingencyLongStitchSewTo = 0xF2,

        ContingencySequinUtilize = 0xF5,
        ContingencySequinJump = 0xF6,
        ContingencySequinStitch = 0xF7,
        ContingencySequinRemove = 0xF8,

        Alternate = 0x100,
    }

    impl From<i32> for Instruction {
        fn from(value: i32) -> Self {
            match value {
                -1 => Instruction::NoInstruction,
                0 => Instruction::Stitch,
                1 => Instruction::Jump,
                2 => Instruction::Trim,
                3 => Instruction::Stop,
                4 => Instruction::End,
                5 => Instruction::ColorChange,
                9 => Instruction::NeedleSet,
                6 => Instruction::SequinMode,
                7 => Instruction::SequinEject,
                0xB => Instruction::Slow,
                0xC => Instruction::Fast,
                0x10 => Instruction::SetChangeSequence,
                0xB0 => Instruction::SewTo,
                0xB1 => Instruction::NeedleAt,
                0xE0 => Instruction::StitchBreak,
                0xE1 => Instruction::SequenceBreak,
                0xE2 => Instruction::ColorBreak,
                0xE4 => Instruction::TieOn,
                0xE5 => Instruction::TieOff,
                0xE9 => Instruction::FrameEject,
                0xC0 => Instruction::MatrixTranslate,
                0xC1 => Instruction::MatrixScaleOrigin,
                0xC2 => Instruction::MatrixRotateOrigin,
                0xC4 => Instruction::MatrixScale,
                0xC5 => Instruction::MatrixRotate,
                0xC3 => Instruction::MatrixReset,
                0xD5 => Instruction::OptionMaxStitchLength,
                0xD6 => Instruction::OptionMaxJumpLength,
                0xD7 => Instruction::OptionExplicitTrim,
                0xD8 => Instruction::OptionImplicitTrim,
                0xD3 => Instruction::ContingencyTieOnNone,
                0xD1 => Instruction::ContingencyTieOnThreeSmall,
                0xD4 => Instruction::ContingencyTieOffNone,
                0xD2 => Instruction::ContingencyTieOffThreeSmall,
                0xF0 => Instruction::ContingencyLongStitchNone,
                0xF1 => Instruction::ContingencyLongStitchJumpNeedle,
                0xF2 => Instruction::ContingencyLongStitchSewTo,
                0xF5 => Instruction::ContingencySequinUtilize,
                0xF6 => Instruction::ContingencySequinJump,
                0xF7 => Instruction::ContingencySequinStitch,
                0xF8 => Instruction::ContingencySequinRemove,
                0x100 => Instruction::Alternate,
                _ => Instruction::NoInstruction, // Default case for unknown values
            }
        }
    }

    /// Stores instruction and coordinates of its appliance
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Stitch {
        /// X coordinate of instruction
        pub x: i32,
        /// Y coordinate of instruction
        pub y: i32,
        /// Instruction, which is applied at the coordinates
        pub instruction: Instruction,
    }
}

crate::mod_interface! {
  own use Instruction;
  own use Stitch;
}

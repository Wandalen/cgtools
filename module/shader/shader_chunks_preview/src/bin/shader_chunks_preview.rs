//! Standalone `shader_chunks_preview` binary — a one-line delegate to the
//! crate's [`shader_chunks_preview::run`] so the standalone and aggregated
//! ( `shader_chunks`/`sch` ) spellings share every byte of behavior.

fn main()
{
  shader_chunks_preview::run();
}

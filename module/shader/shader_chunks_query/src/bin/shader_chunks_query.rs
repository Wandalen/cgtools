//! Standalone `shader_chunks_query` binary — a one-line delegate to the
//! crate's [`shader_chunks_query::run`] so the standalone and aggregated
//! ( `shader_chunks`/`sch` ) spellings share every byte of behavior.

fn main()
{
  shader_chunks_query::run();
}

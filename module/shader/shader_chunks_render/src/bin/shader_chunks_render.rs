//! Standalone binary for the render utility — one-line delegate to
//! [`shader_chunks_render::run`]; the aggregated `shader_chunks`/`sch`
//! binaries serve the same command with identical behavior.

fn main()
{
  shader_chunks_render::run();
}

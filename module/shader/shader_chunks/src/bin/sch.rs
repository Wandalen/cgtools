//! Short-alias binary — one-line delegate to the shared
//! [`shader_chunks::cli`] wiring layer; behaves byte-identically to the
//! primary `shader_chunks` binary.

fn main()
{
  shader_chunks::cli::run();
}

//! Primary binary — one-line delegate to the shared [`shader_chunks::cli`]
//! wiring layer; `sch` is the byte-identical short alias.

fn main()
{
  shader_chunks::cli::run();
}

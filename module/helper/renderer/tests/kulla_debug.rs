use renderer::webgl::loaders::kulla_conty::kulla_conty_tables;

#[ test ]
fn debug_print_eaavg()
{
  let t = kulla_conty_tables( 32, 32, 1024 );
  for r in [ 0usize, 1, 2, 3, 5, 8, 16, 31 ]
  {
    println!( "roughness={:.3} e_avg={:.4}", r as f32 / 31.0, t.e_avg[ r ] );
  }
}

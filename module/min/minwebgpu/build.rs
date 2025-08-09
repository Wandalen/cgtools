//! Build script to enable web_sys unstable APIs for WebGPU support.

fn main() {
  // Enable web_sys unstable APIs for WebGPU support
  println!("cargo:rustc-cfg=web_sys_unstable_apis");
}
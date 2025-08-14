//!

mod private
{
  use minwebgl as gl;

  use gl::F32x4x4;
  use std::{ cell::RefCell, rc::Rc };
  use renderer::webgl::
  {
    Object3D,
    Node,
    Scene
  };

  struct Rigs
  {
    rigs : Vec< ( Rig, Vec< Animation > ) >
  }

  // Scene analog
  pub struct Rig
  {
    // Contains global matrix, local matrix and joints, weights attributes
    root : Rc< RefCell< Node > >,
    joint_matrices : Vec< F32x4x4 >
  }

  impl Rig
  {
    fn a( : Rc< RefCell< Node > > )
    {

    }
  }

  pub struct Animation
  {
    sequencer : Sequencer,
  }

  impl Animation
  {
    fn a( : Rc< RefCell< Node > > )
    {

    }
  }

  pub async fn load
  (
    document : &gl::web_sys::Document,
    gltf_path : &str,
    gl : &gl::WebGl2RenderingContext
  ) 
  -> Result< Rigs, gl::WebglError >
  {

  }
}

crate::mod_interface!
{
  orphan use
  {
    
  };
}
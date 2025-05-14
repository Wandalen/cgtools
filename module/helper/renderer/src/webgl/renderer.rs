mod private
{
  use std::{ cell::RefCell, collections::HashMap, rc::Rc };
  use minwebgl as gl;

  use crate::webgl::
  { 
    Camera, 
    IBL, 
    Object3D,
    Node, 
    ProgramInfo, 
    Scene,
    Primitive,
    AlphaMode
  };

  const MAIN_VERTEX_SHADER : &'static str = include_str!( "shaders/main.vert" );
  const MAIN_FRAGMENT_SHADER : &'static str = include_str!( "shaders/main.frag" );

  #[ derive( Default ) ]
  pub struct Renderer
  {
    /// A list of all glPrograms used
    programs : HashMap< String, ProgramInfo >,
    /// A struct that holds three textures needed for Image Based Lightning
    ibl : IBL,
    /// List of transparent nodes. Filled in runtime
    transparent_nodes : Vec< ( Rc< RefCell< Node > >, Rc< RefCell< Primitive > > ) >
  }

  impl Renderer 
  {
    pub fn new() -> Self
    {
      Self::default()
    } 

    pub fn set_ibl( &mut self, ibl : IBL )
    {
      self.ibl = ibl;
    }

    // pub async fn load_ibl( &mut self, gl : &gl::WebGl2RenderingContext, path : &str )
    // {
    //   self.ibl = loaders::ibl::load( gl, path ).await;
    // }

    pub fn render
    ( 
      &mut self, 
      gl : &gl::WebGl2RenderingContext,
      scene : &mut Scene, 
      camera : &Camera 
    ) -> Result< (), gl::WebglError >
    {
      //scene.update_world_matrix();
      
      self.transparent_nodes.clear();

      let mut draw_node = 
      | 
        node : Rc< RefCell< Node > >
      | -> Result< (), gl::WebglError >
      {
        if let Object3D::Mesh( ref mesh ) = node.borrow().object
        {
          for primitive_rc in mesh.borrow().primitives.iter()
          {
            let primitive = primitive_rc.borrow();
            let material = primitive.material.borrow();
            let geometry = primitive.geometry.borrow();
            let vs_defines = geometry.get_defines();
            let program_id = format!( "{}{}", material.id, vs_defines );

            let program_info = 
            if let Some( ref program_info ) = self.programs.get( &program_id )
            {
             program_info 
            }
            else
            {
              let program = gl::ProgramFromSources::new
              ( 
                &format!( "#version 300 es\n{}\n{}", vs_defines, MAIN_VERTEX_SHADER ), 
                &format!( "#version 300 es\n{}\n{}\n{}", vs_defines, material.get_fragment_defines(), MAIN_FRAGMENT_SHADER ) 
              ).compile_and_link( gl )?;
              let program_info = ProgramInfo::new( gl , program );

              let locations = program_info.get_locations();
              program_info.bind( gl );
              const IBL_BASE_ACTIVE_TEXTURE : u32 = 10;
              self.ibl.bind( gl, IBL_BASE_ACTIVE_TEXTURE );
              material.configure( gl, locations, IBL_BASE_ACTIVE_TEXTURE );
              material.upload( gl, locations )?;

              self.programs.insert( program_id.clone(), program_info );
              self.programs.get( &program_id ).unwrap()
            };

            match material.alpha_mode
            {
              AlphaMode::Blend =>
              {
                self.transparent_nodes.push( ( node.clone(), primitive_rc.clone() ) );
                continue;
              },
              _ => {}
            }

            let locations = program_info.get_locations();

            program_info.bind( gl );
            camera.upload( gl, locations );
            node.borrow().upload( gl, locations );
            primitive.bind( gl );
            primitive.draw( gl );
          }
        } 

        Ok( () )
      };

      scene.traverse( &mut draw_node )?;

      self.transparent_nodes.sort_by( | a, b | 
      {
        let dist1 = camera.get_eye().distance_squared( &a.1.borrow().center() );
        let dist2 = camera.get_eye().distance_squared( &b.1.borrow().center() );

        dist1.partial_cmp( &dist2 ).unwrap()
      });

      for ( node, primitive ) in self.transparent_nodes.iter()
      {
        let primitive = primitive.borrow();
        let material = primitive.material.borrow();
        let geometry = primitive.geometry.borrow();
        let vs_defines = geometry.get_defines();
        let program_info = self.programs.get( &format!( "{}{}",  material.id, vs_defines ) ).unwrap();

        let locations = program_info.get_locations();

        program_info.bind( gl );
        camera.upload( gl, locations );
        node.borrow().upload( gl, locations );
        primitive.bind( gl );
        primitive.draw( gl );
      }

      Ok( () )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Renderer
  };
}
use std::{ cell::RefCell, collections::HashMap, rc::Rc };
use minwebgl as gl;

use crate::
{ 
  camera::Camera, 
  ibl::IBL, 
  loaders, 
  material::Material, 
  mesh::Mesh, 
  node::Node, 
  program::ProgramInfo, 
  scene::Scene
};

const MAIN_VERTEX_SHADER : &'static str = include_str!( "../shaders/main.vert" );
const MAIN_FRAGMENT_SHADER : &'static str = include_str!( "../shaders/main.frag" );


pub struct Renderer< 'a >
{
  /// A list of all materials used by the meshes
  materials : Vec< Material< 'a > >,
  /// A list of all meshes
  meshes : Vec< Mesh >,
  /// A list of all glPrograms used
  programs : Vec< ProgramInfo >,
  /// additional_programs.len() == materials.len() + 1 ( plus one for the default material ).
  /// For every material, using vertex_defines as a key, stores the id into the `programs` array,
  /// specifying which program should be used for this ( vertex_defines, fragment_defines ) pair.
  /// 
  /// Each material is defined by a pair of vertex and fragment defines.
  /// Fragment defines are stored in the material itself, while vertex defines
  /// are stored in primiteves. `glProgram`` is created for each material excluding the vertex defines
  /// and stored in `programs` array. Then additional programs are created for each unique pair ( vertex_defines, fragment_defines ).
  additional_programs : Vec< HashMap< String, usize > >,
  /// Default material according to the Kronos gltf specification
  default_material : Material< 'a >,
  /// A struct that holds three textures needed for Image Based Lightning
  ibl : IBL
}

impl< 'a > Renderer< 'a > 
{
  pub fn new
  ( 
    materials : Vec< Material< 'a > >,
    meshes: Vec< Mesh > 
  ) -> Self
  {
    let programs = Vec::with_capacity( materials.len() * 2 );
    let default_material = Material::default();
    let mut additional_programs = Vec::with_capacity( materials.len() + 1 );
    for _ in 0..materials.len() + 1
    {
      additional_programs.push( HashMap::new() );
    }
    let ibl = Default::default();

    Self
    {
      materials,
      meshes,
      programs,
      default_material,
      additional_programs,
      ibl
    }
  } 

  pub async fn load_ibl( &mut self, gl : &gl::WebGl2RenderingContext, path : &str )
  {
    self.ibl = loaders::ibl::load( gl, path ).await;
  }

  pub fn compile( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
  {
    gl::console::time_with_label( "Compile" );
    let create_program = | vs : &str, fs : &str, m : &Material | -> Result< ProgramInfo, gl::WebglError >
    {
      let program = gl::ProgramFromSources::new( vs, fs ).compile_and_link( &gl )?;
      let program_info = ProgramInfo::new( gl, program );
      program_info.apply( gl );

      m.configure( gl, &program_info );
      m.apply( gl, &program_info )?;
      Ok( program_info )
    };

    let fs_shader_default = format!( "#version 300 es\n{}", MAIN_FRAGMENT_SHADER );
    let vs_shader_default = format!( "#version 300 es\n{}", MAIN_VERTEX_SHADER );

    gl::console::time_with_label( "Compile materials" );
    for m in self.materials.iter()
    {
      let fs_def = m.get_fragment_defines();
      let fs_shader = format!( "#version 300 es\n{}\n{}", fs_def, MAIN_FRAGMENT_SHADER );

      let program_info = create_program( &vs_shader_default, &fs_shader, &m )?;
      self.ibl.bind( gl );
      self.programs.push( program_info );
    }
    gl::console::time_end_with_label( "Compile materials" );

    gl::console::time_with_label( "Compile additional materials" );
    let mut num_additional_programs = 0;
    for m in self.meshes.iter_mut()
    {
      for p in m.primitives.iter_mut()
      {
        let vs_defines = &p.vs_defines;
        if vs_defines.is_empty() { continue }

        let ( mat, m_id ) = if let Some( id ) = p.get_material_id()
        {
          ( &self.materials[ id ], id )
        }
        else
        {
          ( &self.default_material, self.materials.len()  )
        };

        let ap = &mut self.additional_programs[ m_id ];
        if !ap.contains_key( vs_defines.as_str() )
        {
          let fs_defines = mat.get_fragment_defines();
        
          let fs_shader = format!( "#version 300 es\n{}\n{}\n{}", fs_defines, vs_defines, MAIN_FRAGMENT_SHADER );
          let vs_shader = format!( "#version 300 es\n{}\n{}", vs_defines, MAIN_VERTEX_SHADER );

          let program_info = create_program( &vs_shader, &fs_shader, &mat )?;
          self.ibl.bind( gl );

          self.programs.push( program_info );

          ap.insert( vs_defines.clone(), self.programs.len() - 1 );
          num_additional_programs += 1;
        }
      }
    }
    gl::info!( "Number of additional programs: {}", num_additional_programs );
    gl::console::time_end_with_label( "Compile additional materials" );

    let program_info = create_program( &vs_shader_default, &fs_shader_default, &self.default_material )?;
    self.programs.push( program_info );
    gl::console::time_end_with_label( "Compile" );

    Ok( () )
  }

  pub fn render
  ( 
    &self, 
    gl : &gl::WebGl2RenderingContext,
    scene : &mut Scene, 
    camera : &Camera 
  ) -> Result< (), gl::WebglError >
  {
    scene.update_world_matrix();

    for mesh in self.meshes.iter()
    {
      for primitive in mesh.primitives.iter()
      {
        let ( material, m_id ) = if let Some( id ) = primitive.get_material_id() 
        {
          ( &self.materials[ id ], id )
        }
        else
        {
          ( &self.default_material, self.materials.len() )
        };

        let mut program_info = &self.programs[ m_id ];
        let ap = &self.additional_programs[ m_id ];
        if let Some( p_id ) = ap.get( primitive.vs_defines.as_str() )
        {
          program_info = &self.programs[ *p_id ]
        }

        program_info.apply( gl );

        camera.apply( gl, program_info );
        mesh.parent_node.borrow().apply( gl, program_info );
        primitive.apply( gl );
        material.bind_textures( gl );
        primitive.draw( gl );
      }
    }

    Ok( () )
  }
}
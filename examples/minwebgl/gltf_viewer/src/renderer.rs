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
  nodes : Vec< Rc< RefCell< Node > > >,
  materials : Vec< Material< 'a > >,
  meshes : Vec< Mesh >,
  programs : HashMap< uuid::Uuid, ProgramInfo >,
  default_material : Material< 'a >,
  ibl : IBL
}

impl< 'a > Renderer< 'a > 
{
  pub fn new
  ( 
    nodes : Vec< Rc< RefCell< Node > > >,
    materials : Vec< Material< 'a > >,
    meshes: Vec< Mesh > 
  ) -> Self
  {
    let programs = HashMap::new();
    let default_material = Material::default();
    let ibl = Default::default();

    Self
    {
      nodes,
      materials,
      meshes,
      programs,
      default_material,
      ibl
    }
  } 

  pub async fn load_ibl( &mut self, gl : &gl::WebGl2RenderingContext, path : &str )
  {
    self.ibl = loaders::ibl::load( gl, path ).await;
  }

  pub fn compile( &mut self, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
  {
    let vs_shader = format!( "#version 300 es\n{}", MAIN_VERTEX_SHADER );

    for m in self.materials.iter()
    {
      let fs_def = m.get_fragment_defines();
      let fs_shader = format!( "#version 300 es\n{}\n{}", fs_def, MAIN_FRAGMENT_SHADER );

      let program = gl::ProgramFromSources::new( &vs_shader, &fs_shader ).compile_and_link( &gl )?;
      let program_info = ProgramInfo::new( gl, program );
      program_info.apply( gl );

      m.configure( gl, &program_info );
      m.apply( gl, &program_info )?;
      self.ibl.bind( gl );

      self.programs.insert( m.id, program_info );

    }


    let fs_shader = format!( "#version 300 es\n{}", MAIN_FRAGMENT_SHADER );
    let program = gl::ProgramFromSources::new( &vs_shader, &fs_shader ).compile_and_link( &gl )?;
    let program_info = ProgramInfo::new( gl, program );
    program_info.apply( gl );


    self.default_material.apply( gl, &program_info )?;
    self.programs.insert( self.default_material.id, program_info );

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
        let material = if let Some( id ) = primitive.get_material_id() 
        {
          &self.materials[ id ]    
        }
        else
        {
          &self.default_material
        };


        let program_info = &self.programs.get( &material.id ).expect( "Program for the material does not exist" );

        // if program_info.get_locations().get( "normalTexture" ).unwrap().is_none()
        // {
        //   continue;
        // }

        program_info.apply( gl );

        camera.apply( gl, program_info );
        mesh.parent_node.borrow().apply( gl, program_info );
        primitive.apply( gl );
        material.bind_textures( gl );
        primitive.draw( gl );
      }
    }

    // for node in self.nodes.iter()
    // {
    //   let node = node.borrow();
    //   let mesh_id = match node.object 
    //   {
    //       Object3D::Other => continue,
    //       Object3D::Mesh( id ) => id
    //   };

    //   let mesh = &self.meshes[ mesh_id ];
      
    //   for primitive in mesh.primitives.iter()
    //   {
    //     let material = if let Some( id ) = primitive.get_material_id() 
    //     {
    //       &self.materials[ id ]    
    //     }
    //     else
    //     {
    //       &self.default_material
    //     };

    //     let program_info = &self.programs.get( &material.id ).expect( "Program for the material does not exist" );
    //     program_info.apply( gl );

    //     camera.apply( gl, program_info );
    //     node.apply( gl, program_info );
    //     primitive.apply( gl );
    //     material.bind_textures( gl );
    //     primitive.draw( gl );
    //   }
    // }

    Ok( () )
  }
}
/// Internal namespace.
mod private
{
  use crate::*;
  use std::{ collections::HashSet, fmt::Display };
  use tobj::{ Model, Material };
  use web::model::ForBrowser;
  use model::obj;

  impl< 'model, 'mtl > Display for ForBrowser< obj::ReportObjModel< 'model, 'mtl > >
  {
    fn fmt( &self, f: &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      fn format_vec3( v : [ f32; 3 ] ) -> String
      {
        format!
        (
          "( {}, {}, {} )",
          v[ 0 ],
          v[ 1 ],
          v[ 2 ]
        )
      }

      fn format_set< V : Display >( set : &HashSet< V > ) -> String
      {
        let res = set
        .iter()
        .map( | v | v.to_string() )
        .collect::< Vec< _ > >()
        .join( ", " );
        format!( "{{ {} }}", res)
      }

      let box_min = format_vec3( self.report.bounding_box.min.into() );
      let box_max = format_vec3( self.report.bounding_box.max.into() );
      let sphere_center = format_vec3( self.report.bounding_sphere.center.into() );
      let arities_set = format_set( &self.report.num_of_arities );

      write!
      (
        f,
        "\
        === Model Report ===\n\
        Name: {ModelName}\n\
        Memory: {Memory:.2} KB\n\
        Geometry Statistics:\n\
        \x20 • Vertices:       {Vertices}\n\
        \x20 • Normals:        {Normals}\n\
        \x20 • TexCoords:      {TexCoords}\n\
        \x20 • Vertex colors:  {VertexColors}\n\
        \x20 • Faces:          {Faces}\n\
        \x20 • Arities:        {Arities}\n\
        -----------------------------------\n\
        \x20 • Indices:        {Indices}\n\
        \x20 • Texcoords ind.: {Tx_Indicies}\n\
        \x20 • Normals ind.:   {N_Indicies}\n\
        Bounding Volume:\n\
        \x20 • Box:\n\
        \x20    Min: {BoxMin} \n\
        \x20    Max: {BoxMax} \n\
        \x20 • Sphere: \n\
        \x20    Center: {Center} \n\
        \x20    Radius: {Radius}\n\
        ",
        ModelName = self.report.name,
        Memory = self.report.size_in_bytes as f64 / 1024.0,
        Vertices = self.report.num_vertices,
        Normals = self.report.num_normals,
        TexCoords = self.report.num_texcoords,
        VertexColors = self.report.num_vertex_colors,
        Faces = self.report.num_faces,
        Arities = arities_set,
        Indices = self.report.num_indices,
        Tx_Indicies = self.report.num_texcoords_indicies,
        N_Indicies = self.report.num_normal_indicies,
        BoxMin = box_min,
        BoxMax = box_max,
        Center = sphere_center,
        Radius = self.report.bounding_sphere.radius
      )?;

      if self.report.material.is_none()
      {
        write!( f, "Material: None" )
      }
      else
      {
        let m = self.report.material.unwrap().clone();
        let ambient = m.ambient.map_or_else( || String::from( "None" ), | v | format_vec3( v ) );
        let diffuse = m.diffuse.map_or_else( || String::from( "None" ), | v | format_vec3( v ) );
        let specular = m.specular.map_or_else( || String::from( "None" ), | v | format_vec3( v ) );
        let shininess = m.shininess.map_or_else( || String::from( "None" ), | v | v.to_string() );
        let dissolve = m.dissolve.map_or_else( || String::from( "None" ), | v | v.to_string() );
        let optical_density = m.optical_density.map_or_else( || String::from( "None" ), | v | v.to_string() );

        let ambient_texture = m.ambient_texture.map_or_else( || String::from( "None" ), | v | v );
        let diffuse_texture = m.diffuse_texture.map_or_else( || String::from( "None" ), | v | v );
        let specular_texture = m.specular_texture.map_or_else( || String::from( "None" ), | v | v );
        let normal_texture = m.normal_texture.map_or_else( || String::from( "None" ), | v | v );
        let shininess_texture = m.shininess_texture.map_or_else( || String::from( "None" ), | v | v );
        let dissolve_texture = m.dissolve_texture.map_or_else( || String::from( "None" ), | v | v );

        let illumination_model = m.illumination_model.map_or_else( || String::from( "None" ), | v | v.to_string() );
        let unknown_param = format!( "{:#?}", m.unknown_param );

        write!
        (
          f,
          "\
          \x20 • Name: {Name} \n\
          \x20 • Ambient: {Ambient} \n\
          \x20 • Diffuse: {Diffuse} \n\
          \x20 • Specular: {Specular} \n\
          \x20 • Shininess: {Shininess} \n\
          \x20 • Dissolve: {Dissolve} \n\
          \x20 • Optical density: {OptDensity} \n\
          \x20 • Ambient texture: {TAmbient} \n\
          \x20 • Diffuse texture: {TDiffuse} \n\
          \x20 • Specular texture: {TSpecular} \n\
          \x20 • Normal texture: {TNormal} \n\
          \x20 • Shininess texture: {TShininess} \n\
          \x20 • Dissolve texture: {TDissolve} \n\
          \x20 • Illumination model: {IllumModel} \n\
          \x20 • Unknown parameters: {Other} \n\
          ",
          Name = m.name,
          Ambient = ambient,
          Diffuse = diffuse,
          Specular = specular,
          Shininess = shininess,
          Dissolve = dissolve,
          OptDensity = optical_density,
          TAmbient = ambient_texture,
          TDiffuse = diffuse_texture,
          TSpecular = specular_texture,
          TNormal = normal_texture,
          TShininess = shininess_texture,
          TDissolve = dissolve_texture,
          IllumModel = illumination_model,
          Other = unknown_param
        )
      }
    }
  }

  pub fn make_reports< 'model, 'mtl >
  (
    models : &'model [ Model ],
    materials : &'mtl [ Material ]
  )
  -> Vec< ForBrowser< obj::ReportObjModel< 'model, 'mtl > > >
  {
    let reports = obj::make_reports( models, materials );
    ForBrowser::from_reports( reports )
  }

  pub async fn load_model_from_slice
  (
    mut obj_buffer : &[ u8 ],
    material_folder : &str,
    load_options : &tobj::LoadOptions
  )
  -> tobj::LoadResult
  {
    tobj::load_obj_buf_async
    (
      &mut obj_buffer,
      load_options,
      move | p |
      {
        async move {
          let mtl = web::file::load( &format!( "{}/{}", material_folder, p ) ).await;

          if mtl.is_err()
          {
            web::log::error!( "{:#?}", mtl );
            return Err( tobj::LoadError::OpenFileFailed );
          }
          let mtl = mtl.unwrap();
          tobj::load_mtl_buf( &mut mtl.as_ref() )
        }
      }
    )
    .await
  }
}

crate::mod_interface!
{

  orphan use
  {
    make_reports,
    load_model_from_slice
  };

}

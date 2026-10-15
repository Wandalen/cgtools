/// Internal namespace.
mod private
{
  use crate::{ model, web };
  use std::{ collections::HashSet, fmt::Display };
  use tobj::{ Model, Material };
  use web::model::ForBrowser;
  use model::obj;

  #[ inline ]
  fn vec3_format( v : [ f32; 3 ] ) -> String
  {
    format!
    (
      "( {}, {}, {} )",
      v[ 0 ],
      v[ 1 ],
      v[ 2 ]
    )
  }

  #[ inline ]
  fn set_format< V : Display >( set : &HashSet< V > ) -> String
  {
    let res = set
    .iter()
    .map( ToString::to_string )
    .collect::< Vec< _ > >()
    .join( ", " );
    format!( "{{ {res} }}" )
  }

  /// Writes the geometry-statistics and bounding-volume section of a model report.
  #[ inline ]
  fn report_write( f : &mut std::fmt::Formatter< '_ >, report : &obj::ReportObjModel< '_, '_ > ) -> std::fmt::Result
  {
    let box_min = vec3_format( report.bounding_box.min.into() );
    let box_max = vec3_format( report.bounding_box.max.into() );
    let sphere_center = vec3_format( report.bounding_sphere.center.into() );
    let arities_set = set_format( &report.num_of_arities );

    // Model byte counts are far below 2^52 (f64's exact-integer limit) for any
    // realistic 3D model, so this precision loss is immaterial for display purposes.
    let memory_kb = report.size_in_bytes as f64 / 1024.0;

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
      ModelName = report.name,
      Memory = memory_kb,
      Vertices = report.num_vertices,
      Normals = report.num_normals,
      TexCoords = report.num_texcoords,
      VertexColors = report.num_vertex_colors,
      Faces = report.num_faces,
      Arities = arities_set,
      Indices = report.num_indices,
      Tx_Indicies = report.num_texcoords_indicies,
      N_Indicies = report.num_normal_indicies,
      BoxMin = box_min,
      BoxMax = box_max,
      Center = sphere_center,
      Radius = report.bounding_sphere.radius
    )
  }

  /// Writes the material-properties section of a model report.
  #[ inline ]
  fn material_write( f : &mut std::fmt::Formatter< '_ >, m : &Material ) -> std::fmt::Result
  {
    let m = m.clone();
    let ambient = m.ambient.map_or_else( || String::from( "None" ), vec3_format );
    let diffuse = m.diffuse.map_or_else( || String::from( "None" ), vec3_format );
    let specular = m.specular.map_or_else( || String::from( "None" ), vec3_format );
    let shininess = m.shininess.map_or_else( || String::from( "None" ), | v | v.to_string() );
    let dissolve = m.dissolve.map_or_else( || String::from( "None" ), | v | v.to_string() );
    let optical_density = m.optical_density.map_or_else( || String::from( "None" ), | v | v.to_string() );

    let ambient_texture = m.ambient_texture.unwrap_or_else( || String::from( "None" ) );
    let diffuse_texture = m.diffuse_texture.unwrap_or_else( || String::from( "None" ) );
    let specular_texture = m.specular_texture.unwrap_or_else( || String::from( "None" ) );
    let normal_texture = m.normal_texture.unwrap_or_else( || String::from( "None" ) );
    let shininess_texture = m.shininess_texture.unwrap_or_else( || String::from( "None" ) );
    let dissolve_texture = m.dissolve_texture.unwrap_or_else( || String::from( "None" ) );

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

  impl Display for ForBrowser< obj::ReportObjModel< '_, '_ > >
  {
    #[ inline ]
    fn fmt( &self, f: &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      report_write( f, &self.report )?;

      match self.report.material
      {
        Some( m ) => material_write( f, m ),
        None => write!( f, "Material: None" ),
      }
    }
  }

  /// Generates model reports and wraps them for browser-side usage.
  ///
  /// This function first creates detailed `ReportObjModel` instances from the provided
  /// models and materials, and then converts them into a `ForBrowser` format,
  /// likely to facilitate their use in a WebAssembly environment.
  ///
  /// # Arguments
  /// * `models`: A slice of `tobj::Model` instances to be reported on.
  /// * `materials`: A slice of `tobj::Material` instances that the models may reference.
  #[ inline ]
  #[ must_use ]
  pub fn reports_make< 'model, 'mtl >
  (
    models : &'model [ Model ],
    materials : &'mtl [ Material ]
  )
  -> Vec< ForBrowser< obj::ReportObjModel< 'model, 'mtl > > >
  {
    let reports = obj::reports_make( models, materials );
    ForBrowser::from_reports( reports )
  }

  /// Asynchronously loads a 3D model from a byte slice, resolving its materials from a web path.
  ///
  /// This function parses an OBJ model from an in-memory buffer. When it encounters a material
  /// library (`.mtl` file) reference, it asynchronously attempts to fetch that file from the
  /// provided `material_folder` path using web APIs.
  ///
  /// # Arguments
  /// * `obj_buffer`: The byte slice containing the OBJ model data.
  /// * `material_folder`: The base URL or path from which to load material files.
  /// * `load_options`: Configuration options for loading the OBJ model, such as triangulation.
  ///
  /// # Returns
  /// A `tobj::LoadResult` containing the loaded models and materials, or an error if loading fails.
  ///
  /// # Errors
  /// Returns an error if the OBJ buffer cannot be parsed, or if a referenced material
  /// library cannot be fetched or parsed.
  #[ inline ]
  pub async fn model_load_from_slice
  (
    mut obj_buffer : &[ u8 ],
    material_folder : &str,
    load_options : &tobj::LoadOptions
  )
  -> tobj::LoadResult
  {
    // `tobj` deprecated the `async`-feature `load_obj_buf_async` since 4.0.3 in favor of its
    // `futures`/`tokio`-gated variants, but neither applies here: `tobj/tokio` needs tokio's
    // native filesystem support, unavailable on this crate's wasm32 web target; `tobj/futures`
    // would require replacing the `&[u8]` reader with a `futures_lite::AsyncBufRead` adapter,
    // a larger migration than this lint cleanup. `tobj/async`'s deprecated function remains the
    // only currently-wired way to call into tobj's buffer-based async loader.
    #[ expect( deprecated, reason = "tobj/async's deprecated function is the only currently-wired way to call tobj's buffer-based async loader on this crate's wasm32 target — full rationale in the comment above" ) ]
    tobj::load_obj_buf_async
    (
      &mut obj_buffer,
      load_options,
      move | p |
      {
        async move {
          let mtl = web::file::load( &format!( "{material_folder}/{p}" ) ).await;

          let mtl = match mtl
          {
            Ok( mtl ) => mtl,
            Err( e ) =>
            {
              web::log::error!( "{e:#?}" );
              return Err( tobj::LoadError::OpenFileFailed );
            }
          };
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
    reports_make,
    model_load_from_slice
  };

}

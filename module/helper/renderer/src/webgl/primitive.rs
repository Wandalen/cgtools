mod private
{
  use std::{ cell::RefCell, fmt::Debug, rc::Rc };
  use mingl::geometry::BoundingBox;
  use minwebgl as gl;
  use crate::webgl::{ Geometry, Material, MaterialUploadContext };

  /// Represents a renderable object composed of geometry and material.
  #[ derive( Debug ) ]
  pub struct Primitive
  {
    /// The geometry of the primitive.
    pub geometry : Rc< RefCell< Geometry > >,
    /// The material of the primitive.
    pub material : Rc< RefCell< Box< dyn Material > > >
  }

  impl Clone for Primitive
  {
    /// Shares `geometry` (same VAO/buffers) rather than deep-cloning it — `Geometry`
    /// owns raw GL handles behind a `Clone` derive that only copies the JS reference,
    /// not the GPU object, so a struct-level clone would alias the same VAO/buffers
    /// under two independently-refcounted `Rc`s. Every current caller of
    /// `clone_tree`/`Mesh::clone`/`Primitive::clone` wants "place a copy of this
    /// subtree elsewhere in the graph", not an independently-mutable GPU copy, so
    /// sharing is the correct semantics here. `material` is still deep-cloned since
    /// materials do not own raw GL handles directly (only via already-Rc-shared
    /// `TextureInfo`), so an independent instance per placement is safe and matches
    /// existing behaviour (e.g. per-instance material state).
    fn clone( &self ) -> Self
    {
      Self
      {
        geometry : self.geometry.clone(),
        material : Rc::new( RefCell::new( self.material.borrow().dyn_clone() ) )
      }
    }
  }

  impl Primitive
  {
    /// Uploads the material properties and geometry data to the GPU.
    ///
    /// * `gl`: The `WebGl2RenderingContext` to use for uploading.
    /// * `locations`: A hash map of uniform locations in the shader program.
    pub fn upload
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      context : &MaterialUploadContext< '_ >
    ) -> Result< (), gl::WebglError >
    {
      self.material.borrow().upload_on_state_change( gl, context )?;
      self.geometry.borrow().upload( gl )?;

      Ok( () )
    }

    /// Binds the material and geometry for rendering.
    ///
    /// * `gl`: The `WebGl2RenderingContext` to use for binding.
    pub fn bind( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.material.borrow().bind( gl );
      self.geometry.borrow().bind( gl );
    }

    /// Draws the primitive using the currently bound material and geometry.
    ///
    /// * `gl`: The `WebGl2RenderingContext` to use for drawing.
    pub fn draw( &self, gl : &gl::WebGl2RenderingContext )
    {
      self.geometry.borrow().draw( gl );
    }

    /// Returns the center point of the primitive's geometry.
    #[ must_use ]
    pub fn center( &self ) -> gl::F32x3
    {
      self.geometry.borrow().center()
    }
    /// Returns the bounding box of the geometry.
    #[ must_use ]
    pub fn bounding_box( &self ) -> BoundingBox
    {
      self.geometry.borrow().bounding_box()
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Primitive
  };
}

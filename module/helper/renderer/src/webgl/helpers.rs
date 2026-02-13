mod private
{
  use crate::webgl::{ Material };
  use std::cell::{ Ref, RefMut };

  /// Assuming you have the Rc< RefCell< Box< dyn Material > > >. You can pass the result of `borrow` call to this function
  /// and it will cast the material to the specified type( another more specific material ).
  ///
  /// Will panic if cast is not possible
  pub fn cast_unchecked_material_to_ref< T : 'static >( material :  Ref< '_, Box< dyn Material > > ) -> Ref< '_, T >
  {
    Ref::map
    (
      material,
      | r |
      {
        ( r.as_ref() as &dyn std::any::Any  ).downcast_ref::< T >()
        .expect( "Cannot cast the material to the specified type" )
      }
    )
  }

  /// Assuming you have the Rc< RefCell< Box< dyn Material > > >. You can pass the result of `borrow_mut` call to this function
  /// and it will cast the material to the specified type( another more specific material ).
  ///
  /// Will panic if cast is not possible
  pub fn cast_unchecked_material_to_ref_mut< T : 'static >( material : RefMut< '_, Box< dyn Material > > ) -> RefMut< '_, T >
  {
    RefMut::map
    (
      material,
      | r |
      {
        ( r.as_mut() as &mut dyn std::any::Any  ).downcast_mut::< T >()
        .expect( "Cannot cast the material to the specified type" )
      }
    )
  }

}

crate::mod_interface!
{

  orphan use
  {
    cast_unchecked_material_to_ref,
    cast_unchecked_material_to_ref_mut
  };
}

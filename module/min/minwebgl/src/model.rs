mod private
{

}

crate::mod_interface!
{
  /// Utilities related to the model in obj format
  #[ cfg( feature = "model_obj" ) ]
  layer obj;
}

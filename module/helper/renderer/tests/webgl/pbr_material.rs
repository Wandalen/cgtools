use super::the_module;
use the_module::{ CullMode, AlphaMode };

/// Test CullMode default is Back.
#[ test ]
fn test_cull_mode_default()
{
  let mode = CullMode::default();
  assert_eq!( mode, CullMode::Back );
}

/// Test CullMode variants are distinct.
#[ test ]
fn test_cull_mode_variants()
{
  assert_ne!( CullMode::Front, CullMode::Back );
  assert_ne!( CullMode::Back, CullMode::FrontAndBack );
  assert_ne!( CullMode::Front, CullMode::FrontAndBack );
}

/// Test CullMode clone and copy.
#[ test ]
fn test_cull_mode_clone_copy()
{
  let mode = CullMode::Front;
  let cloned = mode.clone();
  let copied = mode;
  assert_eq!( mode, cloned );
  assert_eq!( mode, copied );
}

/// Test Option<CullMode> works as expected for material API.
#[ test ]
fn test_cull_mode_option()
{
  let none : Option< CullMode > = None;
  let some_back = Some( CullMode::Back );
  let some_front = Some( CullMode::Front );

  assert!( none.is_none() );
  assert_eq!( some_back.unwrap(), CullMode::Back );
  assert_ne!( some_front, some_back );
}

/// Test AlphaMode default is Opaque.
#[ test ]
fn test_alpha_mode_default()
{
  assert_eq!( AlphaMode::default(), AlphaMode::Opaque );
}

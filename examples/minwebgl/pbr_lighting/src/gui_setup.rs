
use minwebgl as gl;
use renderer::webgl::{ PointLight, DirectLight, Light, Node, Object3D, Renderer };
use serde::{ Deserialize, Serialize };
use gl::
{
  wasm_bindgen::prelude::*,
  F32x3
};
use std::{ rc::Rc, cell::RefCell };

use crate::lil_gui::
{
  add_slider,
  add_color,
  add_dropdown,
  new_gui,
  on_change,
  on_finish_change,
  show
};

#[ derive( Copy, Clone, Serialize, Deserialize ) ]
pub enum LightMode
{
  Direct,
  Point,
  ControllableDirect,
  ControllablePoint
}

#[ derive( Serialize, Deserialize ) ]
pub struct Settings
{
  #[ serde( rename = "bloomRadius" ) ]
  bloom_radius : f32,
  #[ serde( rename = "bloomStrength" ) ]
  bloom_strength : f32,
  exposure : f32,
  /// Direct/Point/ControllableDirect/ControllablePoint
  #[ serde( rename = "lightMode" ) ]
  pub light_mode : LightMode,
  /// Controllable light pitch
  #[ serde( rename = "lightPitch" ) ]
  pub light_pitch : f32,
  /// Controllable light yaw
  #[ serde( rename = "lightYaw" ) ]
  pub light_yaw : f32,
  /// Controllable light distance to scene center
  #[ serde( rename = "lightDistance" ) ]
  pub light_distance : f32,
  /// Controllable light color (RGB)
  #[ serde( rename = "lightColor" ) ]
  pub light_color : [ f32; 3 ],
  /// Controllable light strength/intensity
  #[ serde( rename = "lightStrength" ) ]
  pub light_strength : f32,
  /// Controllable light range (for point lights only)
  #[ serde( rename = "lightRange" ) ]
  pub light_range : f32,
}

impl Settings
{
  fn get_controllable_light_position( &self ) -> F32x3
  {
    F32x3::from_spherical
    (
      self.light_distance,
      self.light_pitch,
      self.light_yaw
    )
  }
}

impl Default for Settings
{
  fn default() -> Self
  {
    Self
    {
      bloom_radius : 0.0,
      bloom_strength : 0.0,
      exposure : 0.0,
      light_mode : LightMode::Direct,
      light_pitch : 0.0,
      light_yaw : 0.0,
      light_distance : 1.0,
      light_color : [ 1.0, 1.0, 1.0 ],
      light_strength : 10.0,
      light_range : 10.0,
    }
  }
}

/// Fills the controllable-light settings from the light's current parameters.
fn init_settings_from_light( light : &Light, settings : &mut Settings )
{
  match light
  {
    Light::Point( point_light ) =>
    {
      let ( r, pitch, yaw ) = F32x3::to_spherical( point_light.position );
      settings.light_mode = LightMode::ControllablePoint;
      settings.light_distance = r;
      settings.light_pitch = pitch;
      settings.light_yaw = yaw;
      settings.light_strength = point_light.strength;
      settings.light_range = point_light.range;
      settings.light_color = point_light.color.0;
    },
    Light::Direct( direct_light ) =>
    {
      let ( r, pitch, yaw ) = F32x3::to_spherical( direct_light.direction );
      settings.light_mode = LightMode::ControllableDirect;
      settings.light_distance = r;
      settings.light_pitch = pitch;
      settings.light_yaw = yaw;
      settings.light_strength = direct_light.strength;
      settings.light_color = direct_light.color.0;
    }
    Light::Spot( _ ) => {}
  }
}

/// Adds the bloom radius, bloom strength, and exposure sliders wired to the renderer.
fn setup_renderer_sliders( gui : &JsValue, object : &JsValue, renderer : &Rc< RefCell< Renderer > > )
{
  let prop = add_slider( gui, object, "bloomRadius", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let renderer = renderer.clone();
      move | value |
      {
        renderer.borrow_mut().set_bloom_radius( value );
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  let prop = add_slider( gui, object, "bloomStrength", 0.0, 10.0, 0.1 );
  let callback = Closure::new
  (
    {
      let renderer = renderer.clone();
      move | value |
      {
        renderer.borrow_mut().set_bloom_strength( value );
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  let prop = add_slider( gui, object, "exposure", -10.0, 10.0, 0.1 );
  let callback = Closure::new
  (
    {
      let renderer = renderer.clone();
      move | value |
      {
        renderer.borrow_mut().set_exposure( value );
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();
}

/// Zeroes out the controllable light's strength whatever its current variant is.
fn mute_controllable_light( controllable_light : &Rc< RefCell< Node > > )
{
  if let Object3D::Light( light ) = &mut controllable_light.borrow_mut().object
  {
    match light
    {
      Light::Direct( direct ) =>
      {
        direct.strength = 0.0;
      },
      Light::Point( point ) =>
      {
        point.strength = 0.0;
        point.range = 0.0;
      },
      Light::Spot( spot ) =>
      {
        spot.strength = 0.0;
      },
    }
  }
}

/// Sets every animated direct light's strength.
fn set_direct_strengths( directs : &[ Rc< RefCell< Node > > ], strength : f32 )
{
  for direct in directs
  {
    if let Object3D::Light( Light::Direct( direct ) ) = &mut direct.borrow_mut().object
    {
      direct.strength = strength;
    }
  }
}

/// Sets every animated point light's strength and range.
fn set_point_strengths( points : &[ Rc< RefCell< Node > > ], strength : f32, range : f32 )
{
  for point in points
  {
    if let Object3D::Light( Light::Point( point ) ) = &mut point.borrow_mut().object
    {
      point.strength = strength;
      point.range = range;
    }
  }
}

/// Activates the animated direct lights, muting the point rig and the controllable light.
fn apply_direct_mode( points : &[ Rc< RefCell< Node > > ], directs : &[ Rc< RefCell< Node > > ], controllable_light : &Rc< RefCell< Node > > )
{
  mute_controllable_light( controllable_light );
  set_point_strengths( points, 0.0, 0.0 );
  set_direct_strengths( directs, 50.0 );
}

/// Activates the animated point lights, muting the direct rig and the controllable light.
fn apply_point_mode( points : &[ Rc< RefCell< Node > > ], directs : &[ Rc< RefCell< Node > > ], controllable_light : &Rc< RefCell< Node > > )
{
  mute_controllable_light( controllable_light );
  set_direct_strengths( directs, 0.0 );
  set_point_strengths( points, 100.0, 10.0 );
}

/// Switches the controllable light to a direct light built from the current settings, muting the animated rigs.
fn apply_controllable_direct_mode
(
  points : &[ Rc< RefCell< Node > > ],
  directs : &[ Rc< RefCell< Node > > ],
  controllable_light : &Rc< RefCell< Node > >,
  settings : &Rc< RefCell< Settings > >,
)
{
  set_direct_strengths( directs, 0.0 );
  set_point_strengths( points, 0.0, 0.0 );

  if let Object3D::Light( light ) = &mut controllable_light.borrow_mut().object
  {
    *light = Light::Direct
    (
      DirectLight
      {
        direction : settings.borrow().get_controllable_light_position(),
        color : F32x3::from_array( settings.borrow().light_color ),
        strength : settings.borrow().light_strength
      }
    );
  }
}

/// Switches the controllable light to a point light built from the current settings, muting the animated rigs.
fn apply_controllable_point_mode
(
  points : &[ Rc< RefCell< Node > > ],
  directs : &[ Rc< RefCell< Node > > ],
  controllable_light : &Rc< RefCell< Node > >,
  settings : &Rc< RefCell< Settings > >,
)
{
  set_direct_strengths( directs, 0.0 );
  set_point_strengths( points, 0.0, 0.0 );

  if let Object3D::Light( light ) = &mut controllable_light.borrow_mut().object
  {
    *light = Light::Point
    (
      PointLight
      {
        position : settings.borrow().get_controllable_light_position(),
        color : F32x3::from_array( settings.borrow().light_color ),
        strength : settings.borrow().light_strength,
        range : settings.borrow().light_range
      }
    );
  }
}

/// Adds the light mode dropdown that switches between the animated and controllable lighting rigs.
fn setup_light_mode_dropdown
(
  gui : &JsValue,
  object : &JsValue,
  points : Vec< Rc< RefCell< Node > > >,
  directs : Vec< Rc< RefCell< Node > > >,
  controllable_light : &Rc< RefCell< Node > >,
  settings : &Rc< RefCell< Settings > >,
)
{
  let light_modes = vec!
  [
    LightMode::Direct,
    LightMode::Point,
    LightMode::ControllableDirect,
    LightMode::ControllablePoint
  ];

  let prop = add_dropdown
  (
    gui,
    object,
    "lightMode",
    &serde_wasm_bindgen::to_value( light_modes.as_slice() ).unwrap()
  );

  let callback = Closure::new
  (
    {
      let controllable_light = controllable_light.clone();
      let settings = settings.clone();
      move | value : JsValue |
      {
        if let Ok( light_mode ) = serde_wasm_bindgen::from_value::< LightMode >( value )
        {
          settings.borrow_mut().light_mode = light_mode;
          match light_mode
          {
            LightMode::Direct =>
            {
              apply_direct_mode( &points, &directs, &controllable_light );
            },
            LightMode::Point =>
            {
              settings.borrow_mut().light_mode = LightMode::Point;
              apply_point_mode( &points, &directs, &controllable_light );
            },
            LightMode::ControllableDirect =>
            {
              settings.borrow_mut().light_mode = LightMode::ControllableDirect;
              apply_controllable_direct_mode( &points, &directs, &controllable_light, &settings );
            },
            LightMode::ControllablePoint =>
            {
              settings.borrow_mut().light_mode = LightMode::ControllablePoint;
              apply_controllable_point_mode( &points, &directs, &controllable_light, &settings );
            }
          }
        }
      }
    }
  );
  on_finish_change( &prop, &callback );
  callback.forget();
}

/// Adds one spherical-coordinate slider that repositions the controllable light from the settings.
fn add_position_slider
(
  gui : &JsValue,
  object : &JsValue,
  name : &str,
  range : ( f64, f64, f64 ),
  controllable_light : &Rc< RefCell< Node > >,
  settings : &Rc< RefCell< Settings > >,
  field : fn( &mut Settings ) -> &mut f32,
)
{
  let prop = add_slider( gui, object, name, range.0, range.1, range.2 );
  let callback = Closure::new
  (
    {
      let controllable_light = controllable_light.clone();
      let settings = settings.clone();
      move | value : f32 |
      {
        *field( &mut settings.borrow_mut() ) = value;
        let position = settings.borrow().get_controllable_light_position();
        if let Object3D::Light( light ) = &mut controllable_light.borrow_mut().object
        {
          match light
          {
            Light::Direct( direct ) =>
            {
              direct.direction = position;
            },
            Light::Point( point ) =>
            {
              point.position = position;
            },
            Light::Spot( spot ) =>
            {
              spot.position = position;
            },
          }
        }
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();
}

/// Adds the color picker wired to the controllable light's color.
fn add_color_control( gui : &JsValue, object : &JsValue, controllable_light : &Rc< RefCell< Node > >, settings : &Rc< RefCell< Settings > > )
{
  let prop = add_color( gui, object, "lightColor" );
  let callback = Closure::new
  (
    {
      let controllable_light = controllable_light.clone();
      let settings = settings.clone();
      move | value : JsValue |
      {
        if let Ok( color ) = serde_wasm_bindgen::from_value::< [ f32; 3 ] >( value )
        {
          settings.borrow_mut().light_color = color;
          if let Object3D::Light( light ) = &mut controllable_light.borrow_mut().object
          {
            match light
            {
              Light::Direct( direct ) =>
              {
                direct.color = F32x3::from_array( color );
              },
              Light::Point( point ) =>
              {
                point.color = F32x3::from_array( color );
              },
              Light::Spot( spot ) =>
              {
                spot.color = F32x3::from_array( color );
              },
            }
          }
        }
      }
    }
  );
  on_finish_change( &prop, &callback );
  callback.forget();
}

/// Adds the strength slider wired to the controllable light's intensity.
fn add_strength_slider( gui : &JsValue, object : &JsValue, controllable_light : &Rc< RefCell< Node > >, settings : &Rc< RefCell< Settings > > )
{
  let prop = add_slider( gui, object, "lightStrength", 0.0, 1000.0, 1.0 );
  let callback = Closure::new
  (
    {
      let controllable_light = controllable_light.clone();
      let settings = settings.clone();
      move | value : f32 |
      {
        settings.borrow_mut().light_strength = value;
        if let Object3D::Light( light ) = &mut controllable_light.borrow_mut().object
        {
          match light
          {
            Light::Direct( direct ) =>
            {
              direct.strength = value;
            },
            Light::Point( point ) =>
            {
              point.strength = value;
            },
            Light::Spot( spot ) =>
            {
              spot.strength = value;
            },
          }
        }
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();
}

/// Adds the range slider affecting the controllable light when it is a point light.
fn add_range_slider( gui : &JsValue, object : &JsValue, controllable_light : &Rc< RefCell< Node > >, settings : &Rc< RefCell< Settings > > )
{
  let prop = add_slider( gui, object, "lightRange", 0.1, 50.0, 0.1 );
  let callback = Closure::new
  (
    {
      let controllable_light = controllable_light.clone();
      let settings = settings.clone();
      move | value : f32 |
      {
        settings.borrow_mut().light_range = value;
        if let Object3D::Light( Light::Point( point ) ) = &mut controllable_light.borrow_mut().object
        {
          point.range = value;
        }
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();
}

/// Setup UI for PBR lighting example
///
/// Arguments:
///
/// - lights - shared link on animated light sources that can be controlled
/// - controllable_light - shared link on light source with configurable parameters
///
/// Shared link are used to update parameters by UI and animate light sources in main loop
pub fn setup
(
  renderer : &Rc< RefCell< Renderer > >,
  mut lights : Vec< Rc< RefCell< Node > > >,
  controllable_light : &Rc< RefCell< Node > >
)
-> Option< Rc< RefCell< Settings > > >
{
  if lights.iter()
  .any( | n | !matches!( n.borrow().object, Object3D::Light( _ ) ) )
  {
    return None;
  }

  let Object3D::Light( light ) = &controllable_light.borrow().object
  else
  {
    return None;
  };

  let controllable_name = controllable_light.borrow().get_name().unwrap();
  lights.retain( | n | n.borrow().get_name() != Some( controllable_name.clone() ) );
  let points = lights.iter().filter( |& n | matches!( n.borrow().object, Object3D::Light( Light::Point( _ ) ) ) ).cloned()
  .collect::< Vec< _ > >();
  let directs = lights.iter().filter( |& n | matches!( n.borrow().object, Object3D::Light( Light::Direct( _ ) ) ) ).cloned()
  .collect::< Vec< _ > >();

  let mut settings = Settings::default();
  settings.bloom_radius = renderer.borrow().bloom_radius();
  settings.bloom_strength = renderer.borrow().bloom_strength();
  settings.exposure = renderer.borrow().exposure();

  init_settings_from_light( light, &mut settings );

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = new_gui();

  let settings = Rc::new( RefCell::new( settings ) );

  // Bloom and exposure controls
  setup_renderer_sliders( &gui, &object, renderer );

  // Lighting mode
  setup_light_mode_dropdown( &gui, &object, points, directs, controllable_light, &settings );

  // Controllable light pitch slider
  add_position_slider( &gui, &object, "lightPitch", ( 0.0, 360.0, 0.1 ), controllable_light, &settings, | s | &mut s.light_pitch );

  // Controllable light yaw slider
  add_position_slider( &gui, &object, "lightYaw", ( -80.0, 80.0, 0.1 ), controllable_light, &settings, | s | &mut s.light_yaw );

  // Controllable light distance slider
  add_position_slider( &gui, &object, "lightDistance", ( 0.01, 5.0, 0.01 ), controllable_light, &settings, | s | &mut s.light_distance );

  // Controllable light color
  add_color_control( &gui, &object, controllable_light, &settings );

  // Controllable light strength
  add_strength_slider( &gui, &object, controllable_light, &settings );

  // Controllable light range (for point lights)
  add_range_slider( &gui, &object, controllable_light, &settings );

  std::mem::forget( object );

  show( &gui );

  Some( settings )
}

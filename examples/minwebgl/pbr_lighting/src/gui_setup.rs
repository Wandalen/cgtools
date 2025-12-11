#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::field_reassign_with_default ) ]

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
      mingl::Spherical
      {
        radius : self.light_distance,
        theta : self.light_pitch,
        phi : self.light_yaw
      }
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
  renderer : Rc< RefCell< Renderer > >,
  mut lights : Vec< Rc< RefCell< Node > > >,
  controllable_light : Rc< RefCell< Node > >
)
-> Option< Rc< RefCell< Settings > > >
{
  if lights.iter()
  .any( | n | if let Object3D::Light( _ ) = n.borrow().object { false } else { true } )
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
  let points = lights.iter().cloned()
  .filter( | n | if let Object3D::Light( Light::Point( _ ) ) = n.borrow().object { true } else { false } )
  .collect::< Vec< _ > >();
  let directs = lights.iter().cloned()
  .filter( | n | if let Object3D::Light( Light::Direct( _ ) ) = n.borrow().object { true } else { false } )
  .collect::< Vec< _ > >();

  let mut settings = Settings::default();
  settings.bloom_radius = renderer.borrow().get_bloom_radius();
  settings.bloom_strength = renderer.borrow().get_bloom_strength();
  settings.exposure = renderer.borrow().get_exposure();

  match light
  {
    Light::Point( point_light ) =>
    {
      let mingl::Spherical{ radius : r, theta : pitch, phi : yaw } = F32x3::to_spherical( point_light.position );
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
      let mingl::Spherical{ radius : r, theta : pitch, phi : yaw } = F32x3::to_spherical( direct_light.direction );
      settings.light_mode = LightMode::ControllableDirect;
      settings.light_distance = r;
      settings.light_pitch = pitch;
      settings.light_yaw = yaw;
      settings.light_strength = direct_light.strength;
      settings.light_color = direct_light.color.0;
    }
  }

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = new_gui();

  let settings = Rc::new( RefCell::new( settings ) );

  // Bloom and exposure controls
  let prop = add_slider( &gui, &object, "bloomRadius", 0.0, 1.0, 0.01 );
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

  let prop = add_slider( &gui, &object, "bloomStrength", 0.0, 10.0, 0.1 );
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

  let prop = add_slider( &gui, &object, "exposure", -10.0, 10.0, 0.1 );
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

  let light_modes = vec!
  [
    LightMode::Direct,
    LightMode::Point,
    LightMode::ControllableDirect,
    LightMode::ControllablePoint
  ];

  // Lighting mode
  let prop = add_dropdown
  (
    &gui,
    &object,
    "lightMode",
    &serde_wasm_bindgen::to_value( light_modes.as_slice() ).unwrap()
  );

  let callback = Closure::new
  (
    {
      let points = points.clone();
      let directs = directs.clone();
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
                }
              }

              for point in &points
              {
                if let Object3D::Light( light ) = &mut point.borrow_mut().object
                {
                  if let Light::Point( point ) = light
                  {
                    point.strength = 0.0;
                    point.range = 0.0;
                  }
                }
              }

              for direct in &directs
              {
                if let Object3D::Light( light ) = &mut direct.borrow_mut().object
                {
                  if let Light::Direct( direct ) = light
                  {
                    direct.strength = 50.0;
                  }
                }
              }
            },
            LightMode::Point =>
            {
              settings.borrow_mut().light_mode = LightMode::Point;
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
                }
              }

              for direct in &directs
              {
                if let Object3D::Light( light ) = &mut direct.borrow_mut().object
                {
                  if let Light::Direct( direct ) = light
                  {
                    direct.strength = 0.0;
                  }
                }
              }

              for point in &points
              {
                if let Object3D::Light( light ) = &mut point.borrow_mut().object
                {
                  if let Light::Point( point ) = light
                  {
                    point.strength = 100.0;
                    point.range = 10.0;
                  }
                }
              }
            },
            LightMode::ControllableDirect =>
            {
              settings.borrow_mut().light_mode = LightMode::ControllableDirect;

              for direct in &directs
              {
                if let Object3D::Light( light ) = &mut direct.borrow_mut().object
                {
                  if let Light::Direct( direct ) = light
                  {
                    direct.strength = 0.0;
                  }
                }
              }

              for point in &points
              {
                if let Object3D::Light( light ) = &mut point.borrow_mut().object
                {
                  if let Light::Point( point ) = light
                  {
                    point.strength = 0.0;
                    point.range = 0.0;
                  }
                }
              }

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
            },
            LightMode::ControllablePoint =>
            {
              settings.borrow_mut().light_mode = LightMode::ControllablePoint;

              for direct in &directs
              {
                if let Object3D::Light( light ) = &mut direct.borrow_mut().object
                {
                  if let Light::Direct( direct ) = light
                  {
                    direct.strength = 0.0;
                  }
                }
              }

              for point in &points
              {
                if let Object3D::Light( light ) = &mut point.borrow_mut().object
                {
                  if let Light::Point( point ) = light
                  {
                    point.strength = 0.0;
                    point.range = 0.0;
                  }
                }
              }

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
          }
        }
      }
    }
  );
  on_finish_change( &prop, &callback );
  callback.forget();

  // Controllable light pitch slider
  let prop = add_slider( &gui, &object, "lightPitch", 0.0, 360.0, 0.1 );
  let callback = Closure::new
  (
    {
      let controllable_light = controllable_light.clone();
      let settings = settings.clone();
      move | value : f32 |
      {
        settings.borrow_mut().light_pitch = value;
        if let Object3D::Light( light ) = &mut controllable_light.borrow_mut().object
        {
          match light
          {
            Light::Direct( direct ) =>
            {
              direct.direction = settings.borrow().get_controllable_light_position();
            },
            Light::Point( point ) =>
            {
              point.position = settings.borrow().get_controllable_light_position();
            },
          }
        }
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  // Controllable light yaw slider
  let prop = add_slider( &gui, &object, "lightYaw", -80.0, 80.0, 0.1 );
  let callback = Closure::new
  (
    {
      let controllable_light = controllable_light.clone();
      let settings = settings.clone();
      move | value : f32 |
      {
        settings.borrow_mut().light_yaw = value;
        if let Object3D::Light( light ) = &mut controllable_light.borrow_mut().object
        {
          match light
          {
            Light::Direct( direct ) =>
            {
              direct.direction = settings.borrow().get_controllable_light_position();
            },
            Light::Point( point ) =>
            {
              point.position = settings.borrow().get_controllable_light_position();
            },
          }
        }
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  // Controllable light distance slider
  let prop = add_slider( &gui, &object, "lightDistance", 0.01, 5.0, 0.01 );
  let callback = Closure::new
  (
    {
      let controllable_light = controllable_light.clone();
      let settings = settings.clone();
      move | value : f32 |
      {
        settings.borrow_mut().light_distance = value;
        if let Object3D::Light( light ) = &mut controllable_light.borrow_mut().object
        {
          match light
          {
            Light::Direct( direct ) =>
            {
              direct.direction = settings.borrow().get_controllable_light_position();
            },
            Light::Point( point ) =>
            {
              point.position = settings.borrow().get_controllable_light_position();
            },
          }
        }
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  // Controllable light color
  let prop = add_color( &gui, &object, "lightColor" );
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
            }
          }
        }
      }
    }
  );
  on_finish_change( &prop, &callback );
  callback.forget();

  // Controllable light strength
  let prop = add_slider( &gui, &object, "lightStrength", 0.0, 1000.0, 1.0 );
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
          }
        }
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  // Controllable light range (for point lights)
  let prop = add_slider( &gui, &object, "lightRange", 0.1, 50.0, 0.1 );
  let callback = Closure::new
  (
    {
      let controllable_light = controllable_light.clone();
      let settings = settings.clone();
      move | value : f32 |
      {
        settings.borrow_mut().light_range = value;
        if let Object3D::Light( light ) = &mut controllable_light.borrow_mut().object
        {
          if let Light::Point( point ) = light
          {
            point.range = value;
          }
        }
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  std::mem::forget( object );

  show( &gui );

  Some( settings )
}

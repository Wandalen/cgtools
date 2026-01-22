import GUI from 'https://cdn.jsdelivr.net/npm/lil-gui@0.19/+esm';

export function newGui() 
{
  return new GUI();
}

export function addFolder( gui, name ) 
{
  return gui.addFolder( name );
}

export function addSliderController( gui, object, property, min, max, step ) 
{
  return gui.add(object, property, min, max, step);
}

export function addDropdownController( gui, object, property, options ) 
{
  return gui.add( object, property, options );
}

export function onFinishChange( gui, callback ) 
{
  return gui.onFinishChange( callback );
}

export function onChange( gui, callback ) 
{
  return gui.onChange( callback );
}

export function set_name( gui, name ) 
{
  return gui.name( name );
}

export function hide( gui ) 
{
  return gui.hide();
}

export function show( gui ) 
{
  return gui.show();
}

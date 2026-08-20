import GUI from 'https://cdn.jsdelivr.net/npm/lil-gui@0.19/+esm';

export function newGui()
{
  return new GUI();
}

export function addSliderController( gui, object, property, min, max, step )
{
  return gui.add( object, property, min, max, step );
}

export function addColorController( gui, object, property )
{
  return gui.addColor( object, property );
}

export function onChange( gui, callback )
{
  return gui.onChange( callback );
}

export function onFinishChange( gui, callback )
{
  return gui.onFinishChange( callback );
}

export function show( gui )
{
  return gui.show();
}

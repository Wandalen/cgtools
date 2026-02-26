import GUI from 'https://cdn.jsdelivr.net/npm/lil-gui@0.19/+esm';

export function newGui() {
  return new GUI();
}

export function addFolder(gui, name) {
  return gui.addFolder(name);
}

export function addController(gui, object, property, min, max, step) {
  return gui.add(object, property, min, max, step);
}

export function addColorController(gui, object, property) {
  return gui.addColor(object, property);
}

export function addDropdownController(gui, object, property, options) {
  return gui.add(object, property, options);
}

export function onFinishChange(gui, callback) {
  gui.onFinishChange(event => callback(event.object));
}

export function getTitle(gui) {
  return gui._title || "";
}

export function getFolders(gui) {
  return gui.folders || []
}

export function hide(gui) {
  return gui.hide();
}

export function show(gui) {
  return gui.show();
}

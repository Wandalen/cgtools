// Minimal slider control panel for the preview runner's tunable uniforms:
// addSlider() creates one labeled range input, onChange() registers the
// single callback fired (with every slider's current value, keyed by
// property) whenever any slider moves. No dropdown/show/hide -- every
// bundle parameter is a numeric range and the panel is always visible (see
// index.html / style.css) -- ported from examples/minwebgl/filters/
// controls.js, trimmed to what this runner actually uses.
//
// Also hosts the Shadertoy-style live source editor: initEditor() seeds the
// textarea and wires a debounced input listener, onEdit() registers the
// callback that listener fires, and setDiagnostics()/clearDiagnostics()
// show or hide the last compile/pipeline error underneath it. setChunkTitle()
// sets the page title and on-page heading to the previewed chunk's name.

let sliders = [];
let changeCallback = null;

// Set the page title and the on-page heading to the previewed chunk's name,
// so it's always clear which chunk a given preview session is showing.
export function setChunkTitle(name) {
  document.title = `${name} — Shader Chunk Preview`;
  const el = document.getElementById('chunk-title');
  if (el) el.textContent = name;
}

let editorTextarea = null;
let editCallback = null;
let editDebounceTimer = null;
const EDIT_DEBOUNCE_MS = 500;

// The composed bundle's dependency/harness text, frozen at load -- only the
// target chunk's own text (in editorTextarea) is ever live-edited. Toggling
// their visibility never changes what compiles; see fullSourceGet().
let depsText = '';
let harnessText = '';

export function addSlider(label, property, value, min, max, step) {
  const container = document.getElementById('controls-container');
  if (!container) return;

  const group = document.createElement('div');
  group.className = 'control-group';

  const labelEl = document.createElement('div');
  labelEl.className = 'control-label';
  labelEl.textContent = label;

  const sliderContainer = document.createElement('div');
  sliderContainer.className = 'control-slider-container';

  const slider = document.createElement('input');
  slider.type = 'range';
  slider.className = 'control-slider';
  slider.min = min;
  slider.max = max;
  slider.step = step;
  slider.value = value;
  slider.dataset.property = property;

  const valueDisplay = document.createElement('div');
  valueDisplay.className = 'control-value';
  valueDisplay.textContent = formatValue(value);

  slider.addEventListener('input', (e) => {
    const val = parseFloat(e.target.value);
    valueDisplay.textContent = formatValue(val);
    if (changeCallback) {
      changeCallback(getValues());
    }
  });

  sliderContainer.appendChild(slider);
  sliderContainer.appendChild(valueDisplay);
  group.appendChild(labelEl);
  group.appendChild(sliderContainer);
  container.appendChild(group);

  sliders.push({ property, element: slider });
}

// Set the callback fired with the full current value set on every slider move.
export function onChange(callback) {
  changeCallback = callback;
}

function getValues() {
  const values = {};
  sliders.forEach((s) => {
    values[s.property] = parseFloat(s.element.value);
  });
  return values;
}

// Show up to 3 decimal places, removing trailing zeros.
function formatValue(value) {
  if (Number.isInteger(value)) {
    return value.toString();
  }
  return parseFloat(value.toFixed(3)).toString();
}

// Indent/outdent every line a non-collapsed selection touches by one step (2 spaces),
// matching every mainstream editor's Tab-with-selection convention. Used only for that case --
// collapsed-cursor Tab/Shift+Tab (see the keydown listener below) instead inserts/removes right
// at the cursor, since there's no line span here that isn't just wherever the cursor sits.
function blockIndent(value, start, end, outdent) {
  const blockStart = value.lastIndexOf('\n', start - 1) + 1;
  // A selection ending exactly at a line start (e.g. a triple-click selecting whole lines)
  // must not pull the next, untouched line into the block.
  const atLineStart = end > blockStart && value[end - 1] === '\n';
  let blockEnd = atLineStart ? end : value.indexOf('\n', end);
  if (blockEnd === -1) blockEnd = value.length;

  let firstLineDelta = 0;
  const newLines = value.slice(blockStart, blockEnd).split('\n').map((line, i) => {
    if (outdent) {
      const removable = line.match(/^ {1,2}/);
      const removedLen = removable ? removable[0].length : 0;
      if (i === 0) firstLineDelta = -removedLen;
      return removable ? line.slice(removedLen) : line;
    }
    if (i === 0) firstLineDelta = 2;
    return '  ' + line;
  });
  const newBlock = newLines.join('\n');
  return {
    value: value.slice(0, blockStart) + newBlock + value.slice(blockEnd),
    selectionStart: Math.max(blockStart, start + firstLineDelta),
    selectionEnd: blockStart + newBlock.length,
  };
}

// Splits composed WGSL into labeled sections by its banner comments, in the
// fixed order shader_chunks_preview_core::bundle_build always produces: zero
// or more "dependency chunk" blocks, exactly one "previewing" (target)
// block, then an optional "auto-generated preview harness" block. A line
// that isn't a recognized banner -- including, defensively, stray text
// before the first banner, which should never happen for a well-formed
// bundle -- stays with whichever section it trails, classified 'target' by
// default so nothing is ever silently dropped from what gets compiled.
function sectionsSplit(fullSource) {
  const blocks = [];
  let current = null;
  for (const line of fullSource.split('\n')) {
    if (line.startsWith('// ==== ')) {
      current = { kind: bannerClassify(line), lines: [line] };
      blocks.push(current);
    } else if (current) {
      current.lines.push(line);
    } else {
      current = { kind: 'target', lines: [line] };
      blocks.push(current);
    }
  }
  return blocks.map((b) => ({ kind: b.kind, text: b.lines.join('\n') }));
}

function bannerClassify(bannerLine) {
  if (bannerLine.startsWith('// ==== dependency chunk:')) return 'dependency';
  if (bannerLine.startsWith('// ==== auto-generated preview harness')) return 'harness';
  return 'target';
}

// Reassembles the full, compilable WGSL from the frozen dependency/harness
// text plus the target editor's current (possibly edited) value -- every
// recompile always compiles the whole composed shader, regardless of which
// sections are currently toggled visible.
function fullSourceGet() {
  return [depsText, editorTextarea.value, harnessText].filter((s) => s.length > 0).join('\n\n');
}

// Populates one read-only reference panel (dependencies or harness) with
// `text` and wires its toggle checkbox to show/hide it. When `text` is empty
// (a leaf chunk with no dependencies, or a fragment chunk with no
// synthesized harness) the toggle itself is hidden too, rather than leaving
// a control with nothing to reveal.
//
// Syncs panel.hidden to the checkbox's actual .checked value immediately,
// rather than trusting the textarea's static `hidden` HTML attribute: on a
// same-document reload (F5), Firefox/Chromium restore a checkbox's checked
// state from history without dispatching a `change` event, which would
// otherwise leave the panel stuck hidden behind a visibly-checked box.
function editorPanelWire(panelId, toggleId, text) {
  const panel = document.getElementById(panelId);
  const toggle = document.getElementById(toggleId);
  if (!panel || !toggle) return;
  if (text.length === 0) {
    const label = toggle.closest('label');
    if (label) label.hidden = true;
    return;
  }
  panel.value = text;
  panel.hidden = !toggle.checked;
  toggle.addEventListener('change', () => {
    panel.hidden = !toggle.checked;
  });
}

// Seed the source textarea and wire its debounced input listener. Safe to
// call once at startup; a missing #shader-source element is a no-op, same
// guard style as addSlider().
export function initEditor(initialSource) {
  const textarea = document.getElementById('shader-source');
  if (!textarea) return;

  const sections = sectionsSplit(initialSource);
  depsText = sections.filter((s) => s.kind === 'dependency').map((s) => s.text).join('\n\n');
  harnessText = sections.filter((s) => s.kind === 'harness').map((s) => s.text).join('\n\n');
  const targetText = sections.filter((s) => s.kind === 'target').map((s) => s.text).join('\n\n');

  textarea.value = targetText;
  editorTextarea = textarea;
  editorPanelWire('shader-deps', 'toggle-deps', depsText);
  editorPanelWire('shader-harness', 'toggle-harness', harnessText);

  textarea.addEventListener('input', () => {
    if (editDebounceTimer) clearTimeout(editDebounceTimer);
    editDebounceTimer = setTimeout(() => {
      if (editCallback) editCallback(fullSourceGet());
    }, EDIT_DEBOUNCE_MS);
  });

  // Plain <textarea> Tab/Shift+Tab moves focus off the field instead of
  // indenting -- fatal for a code editor, since every line of WGSL here uses
  // 2-space indentation. Splice indentation in at the cursor/selection
  // ourselves and dispatch a synthetic 'input' event so the debounced
  // recompile listener above still picks up the change (programmatic
  // `.value` writes don't fire native 'input' events on their own).
  textarea.addEventListener('keydown', (e) => {
    // Only plain Tab/Shift+Tab is an indent request -- Ctrl/Alt/Meta+Tab are
    // browser tab-switch and OS window-switch shortcuts (`e.key` reports
    // "Tab" for those too; only the modifier flags distinguish them) and
    // must reach the browser/OS untouched.
    if (e.key !== 'Tab' || e.ctrlKey || e.altKey || e.metaKey) return;
    e.preventDefault();
    const { value } = textarea;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    if (start !== end) {
      // A selection is active: indent/outdent every line it touches. Tab must never replace
      // the selected text with a raw insertion -- that would silently delete the user's code.
      const result = blockIndent(value, start, end, e.shiftKey);
      textarea.value = result.value;
      textarea.selectionStart = result.selectionStart;
      textarea.selectionEnd = result.selectionEnd;
    } else if (e.shiftKey) {
      const lineStart = value.lastIndexOf('\n', start - 1) + 1;
      const removable = value.slice(lineStart, lineStart + 2).match(/^ {1,2}/);
      if (!removable) return;
      const removedLen = removable[0].length;
      textarea.value = value.slice(0, lineStart) + value.slice(lineStart + removedLen);
      const shift = (pos) => Math.max(lineStart, pos - removedLen);
      textarea.selectionStart = shift(start);
      textarea.selectionEnd = shift(end);
    } else {
      textarea.value = value.slice(0, start) + '  ' + value.slice(end);
      textarea.selectionStart = textarea.selectionEnd = start + 2;
    }
    textarea.dispatchEvent(new Event('input', { bubbles: true }));
  });
}

// Set the callback fired (with the textarea's current text) ~500ms after
// the last keystroke.
export function onEdit(callback) {
  editCallback = callback;
}

// Show a compile/pipeline diagnostic message under the editor.
export function setDiagnostics(text) {
  const el = document.getElementById('shader-diagnostics');
  if (!el) return;
  el.textContent = text;
  el.hidden = false;
}

// Hide the diagnostics panel -- called once a recompile succeeds.
export function clearDiagnostics() {
  const el = document.getElementById('shader-diagnostics');
  if (!el) return;
  el.textContent = '';
  el.hidden = true;
}

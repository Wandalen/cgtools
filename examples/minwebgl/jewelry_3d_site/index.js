const exploreView = document.querySelector('.cam-view-3')
const sidebar = document.querySelector('.side-bar')
const headerContainer = document.querySelector('.header--container')
const camView1 = document.querySelector('.cam-view-1')
const camView2 = document.querySelector('.cam-view-2')
const camView3 = document.querySelector('.cam-view-3')

const exitContainer = document.querySelector('.exit--container')
const footerMenu = document.querySelector('.footer--menu')
const configMaterial = document.querySelector('.config--material')
const configGem = document.querySelector('.config--gem')
const closeConfigMaterial = document.querySelector('.close-materials')
const configRing = document.querySelector('.config--ring')
const closeConfigGem = document.querySelector('.close-gems')
const closeConfigRing = document.querySelector('.close-rings')
const footerContainer = document.querySelector('.footer--container')
const gemMenu = document.querySelector('.gem--menu')
const materialsMenu = document.querySelector('.materials--menu')
const ringsMenu = document.querySelector('.rings--menu')

let colorControlsEnabled = false
let firstLoad = true;
let skipScrollAnimation = true;

// Threshold-based scroll configuration
const SCROLL_THRESHOLD = 100; // deltaY threshold to trigger section transition

// Section indices
const SECTION_HERO = 0;
const SECTION_BRILLIANT = 1;
const SECTION_CHOOSE = 2;

let currentSection = SECTION_HERO;
let isAnimating = false;
let accumulatedDelta = 0;

if (history.scrollRestoration) {
  history.scrollRestoration = 'manual';
}

// Configuration state
export let uiState =
{
  gem: "white",
  metal: "silver",
  ring: 0,
  state: "hero",
  position: [0.6373576, 1.1441559, -0.9127405],
  target: [0.55595696, 0.55741394, -1.0331136],
  transitionAnimationEnabled: false,
  gemCustomColor: [1.0, 1.0, 1.0],
  gemMultiplier: 1.0,
  metalCustomColor: [0.753, 0.753, 0.753],
  metalMultiplier: 1.2,
  changed:
    [
      "gem",
      "metal",
      "ring",
      "state"
    ]
};

let isRendererLoaded = false

/**
 * Animates transition from Hero section to Brilliant section
 */
function animateHeroToBrilliant() {
  if (isAnimating || currentSection === SECTION_BRILLIANT) return;
  isAnimating = true;

  const timeline = gsap.timeline({
    onComplete: () => {
      currentSection = SECTION_BRILLIANT;
      isAnimating = false;
      accumulatedDelta = 0;
    }
  });

  // Hide hero content
  timeline.to('.hero--scroller', { opacity: 0, y: '150%', duration: 0.8, ease: "power4.out" })
  timeline.to('.hero--container', { opacity: 0, xPercent: 100, duration: 1.2, ease: "power4.out" }, '-=0.8')

  // Show brilliant content
  timeline.to('.brilliant--text-bg', { opacity: 0.1, duration: 1.2, ease: "power4.inOut" }, '-=1.2')
  timeline.fromTo('.brilliant--container',
    { opacity: 0, x: '-110%' },
    { opacity: 1, x: '0%', duration: 1.2, ease: "power4.inOut" }, '-=1.2')

  // Animate camera
  timeline.to(uiState.position, {
    0: -0.40259048, 1: 2.6242757, 2: -0.18104002,
    duration: 1.2,
    ease: "power4.inOut",
    onUpdate: () => { !uiState.changed.includes("position") && uiState.changed.push("position") }
  }, '-=1.2')

  timeline.to(uiState.target, {
    0: -0.23794234, 1: 0.49070162, 2: -0.32702705,
    duration: 1.2,
    ease: "power4.inOut",
    onUpdate: () => { !uiState.changed.includes("target") && uiState.changed.push("target") }
  }, '-=1.2')

  // Update sidebar indicators
  timeline.to('.side-bar .unique', { opacity: 0.5, scale: 1, duration: 0.8, ease: "power4.inOut" }, '-=1.2')
  timeline.to('.side-bar .brilliant', { opacity: 1, scale: 1.5, duration: 0.8, ease: "power4.inOut" }, '-=0.8')
}

/**
 * Animates transition from Brilliant section to Hero section
 */
function animateBrilliantToHero() {
  if (isAnimating || currentSection === SECTION_HERO) return;
  isAnimating = true;

  const timeline = gsap.timeline({
    onComplete: () => {
      currentSection = SECTION_HERO;
      isAnimating = false;
      accumulatedDelta = 0;
    }
  });

  // Hide brilliant content
  timeline.to('.brilliant--container', { opacity: 0, x: '-110%', duration: 1.2, ease: "power4.inOut" })
  timeline.to('.brilliant--text-bg', { opacity: 0, duration: 1.2, ease: "power4.inOut" }, '-=1.2')

  // Animate camera first
  timeline.to(uiState.position, {
    0: 0.6858612, 1: 2.7440538, 2: -0.026622068,
    duration: 1.2,
    ease: "power4.inOut",
    onUpdate: () => { !uiState.changed.includes("position") && uiState.changed.push("position") }
  }, '-=1.2')

  timeline.to(uiState.target, {
    0: 0.36420232, 1: 0.8480059, 2: -0.36873266,
    duration: 1.2,
    ease: "power4.inOut",
    onUpdate: () => { !uiState.changed.includes("target") && uiState.changed.push("target") }
  }, '-=1.2')

  // Show hero content AFTER camera movement (near the end)
  timeline.to('.hero--container', { opacity: 1, xPercent: 0, duration: 0.8, ease: "power4.out" }, '-=0.5')
  timeline.to('.hero--scroller', { opacity: 1, y: '0%', duration: 0.6, ease: "power4.inOut" }, '-=0.6')

  // Update sidebar indicators
  timeline.to('.side-bar .brilliant', { opacity: 0.5, scale: 1, duration: 0.8, ease: "power4.inOut" }, '-=1.2')
  timeline.to('.side-bar .unique', { opacity: 1, scale: 1.5, duration: 0.8, ease: "power4.inOut" }, '-=0.8')
}

/**
 * Animates transition from Brilliant section to Choose section
 */
function animateBrilliantToChoose() {
  if (isAnimating || currentSection === SECTION_CHOOSE) return;
  isAnimating = true;

  const timeline = gsap.timeline({
    onComplete: () => {
      currentSection = SECTION_CHOOSE;
      isAnimating = false;
      accumulatedDelta = 0;
    }
  });

  // Hide brilliant content
  timeline.to('.brilliant--container', { opacity: 0, x: '-110%', duration: 1.2, ease: "power4.inOut" })
  timeline.to('.brilliant--text-bg', { opacity: 0, duration: 1.2, ease: "power4.inOut" }, '-=1.2')

  // Show choose content - animate from side (not diagonal)
  timeline.fromTo('.choose--text-bg',
    { opacity: 0, x: '200%' },
    { opacity: 0.1, x: '0%', duration: 1.2, ease: "power4.inOut" }, '-=1.2')
  timeline.fromTo('.choose--content',
    { opacity: 0, x: '200%' },
    { opacity: 1, x: '0%', duration: 1.2, ease: "power4.inOut" }, '-=1.2')

  // Animate camera
  timeline.to(uiState.position, {
    0: -0.39456308, 1: 2.431139, 2: 0.23367776,
    duration: 1.2,
    ease: "power4.inOut",
    onUpdate: () => { !uiState.changed.includes("position") && uiState.changed.push("position") }
  }, '-=1.2')

  timeline.to(uiState.target, {
    0: 0.2921338, 1: 0.9732934, 2: -0.18001612,
    duration: 1.2,
    ease: "power4.inOut",
    onUpdate: () => { !uiState.changed.includes("target") && uiState.changed.push("target") }
  }, '-=1.2')

  // Update sidebar indicators
  timeline.to('.side-bar .brilliant', { opacity: 0.5, scale: 1, duration: 0.8, ease: "power4.inOut" }, '-=1.2')
  timeline.to('.side-bar .choose', { opacity: 1, scale: 1.5, duration: 0.8, ease: "power4.inOut" }, '-=0.8')
}

/**
 * Animates transition from Choose section to Brilliant section
 */
function animateChooseToBrilliant() {
  if (isAnimating || currentSection === SECTION_BRILLIANT) return;
  isAnimating = true;

  const timeline = gsap.timeline({
    onComplete: () => {
      currentSection = SECTION_BRILLIANT;
      isAnimating = false;
      accumulatedDelta = 0;
    }
  });

  // Hide choose content - move to side (not diagonal)
  timeline.to('.choose--content', { opacity: 0, x: '200%', duration: 1.2, ease: "power4.inOut" })
  timeline.to('.choose--text-bg', { opacity: 0, x: '200%', duration: 1.2, ease: "power4.inOut" }, '-=1.2')

  // Show brilliant content
  timeline.to('.brilliant--container', { opacity: 1, x: '0%', duration: 1.2, ease: "power4.inOut" }, '-=1.2')
  timeline.to('.brilliant--text-bg', { opacity: 0.1, duration: 1.2, ease: "power4.inOut" }, '-=1.2')

  // Animate camera
  timeline.to(uiState.position, {
    0: -0.40259048, 1: 2.6242757, 2: -0.18104002,
    duration: 1.2,
    ease: "power4.inOut",
    onUpdate: () => { !uiState.changed.includes("position") && uiState.changed.push("position") }
  }, '-=1.2')

  timeline.to(uiState.target, {
    0: -0.23794234, 1: 0.49070162, 2: -0.32702705,
    duration: 1.2,
    ease: "power4.inOut",
    onUpdate: () => { !uiState.changed.includes("target") && uiState.changed.push("target") }
  }, '-=1.2')

  // Update sidebar indicators
  timeline.to('.side-bar .choose', { opacity: 0.5, scale: 1, duration: 0.8, ease: "power4.inOut" }, '-=1.2')
  timeline.to('.side-bar .brilliant', { opacity: 1, scale: 1.5, duration: 0.8, ease: "power4.inOut" }, '-=0.8')
}

/**
 * Animates transition from Hero section to Choose section (direct)
 */
function animateHeroToChoose() {
  if (isAnimating || currentSection === SECTION_CHOOSE) return;
  isAnimating = true;

  const timeline = gsap.timeline({
    onComplete: () => {
      currentSection = SECTION_CHOOSE;
      isAnimating = false;
      accumulatedDelta = 0;
    }
  });

  // Hide hero content
  timeline.to('.hero--scroller', { opacity: 0, y: '150%', duration: 0.8, ease: "power4.out" })
  timeline.to('.hero--container', { opacity: 0, xPercent: 100, duration: 1.2, ease: "power4.out" }, '-=0.8')

  // Show choose content - animate from side (not diagonal)
  timeline.fromTo('.choose--text-bg',
    { opacity: 0, x: '200%' },
    { opacity: 0.1, x: '0%', duration: 1.2, ease: "power4.inOut" }, '-=1.2')
  timeline.fromTo('.choose--content',
    { opacity: 0, x: '200%' },
    { opacity: 1, x: '0%', duration: 1.2, ease: "power4.inOut" }, '-=1.2')

  // Animate camera
  timeline.to(uiState.position, {
    0: -0.39456308, 1: 2.431139, 2: 0.23367776,
    duration: 1.2,
    ease: "power4.inOut",
    onUpdate: () => { !uiState.changed.includes("position") && uiState.changed.push("position") }
  }, '-=1.2')

  timeline.to(uiState.target, {
    0: 0.2921338, 1: 0.9732934, 2: -0.18001612,
    duration: 1.2,
    ease: "power4.inOut",
    onUpdate: () => { !uiState.changed.includes("target") && uiState.changed.push("target") }
  }, '-=1.2')

  // Update sidebar indicators
  timeline.to('.side-bar .unique', { opacity: 0.5, scale: 1, duration: 0.8, ease: "power4.inOut" }, '-=1.2')
  timeline.to('.side-bar .choose', { opacity: 1, scale: 1.5, duration: 0.8, ease: "power4.inOut" }, '-=0.8')
}

/**
 * Animates transition from Choose section to Hero section (direct)
 */
function animateChooseToHero() {
  if (isAnimating || currentSection === SECTION_HERO) return;
  isAnimating = true;

  const timeline = gsap.timeline({
    onComplete: () => {
      currentSection = SECTION_HERO;
      isAnimating = false;
      accumulatedDelta = 0;
    }
  });

  // Hide choose content - move to side (not diagonal)
  timeline.to('.choose--content', { opacity: 0, x: '200%', duration: 1.2, ease: "power4.inOut" })
  timeline.to('.choose--text-bg', { opacity: 0, x: '200%', duration: 1.2, ease: "power4.inOut" }, '-=1.2')

  // Animate camera
  timeline.to(uiState.position, {
    0: 0.6858612, 1: 2.7440538, 2: -0.026622068,
    duration: 1.2,
    ease: "power4.inOut",
    onUpdate: () => { !uiState.changed.includes("position") && uiState.changed.push("position") }
  }, '-=1.2')

  timeline.to(uiState.target, {
    0: 0.36420232, 1: 0.8480059, 2: -0.36873266,
    duration: 1.2,
    ease: "power4.inOut",
    onUpdate: () => { !uiState.changed.includes("target") && uiState.changed.push("target") }
  }, '-=1.2')

  // Show hero content AFTER choose is hidden (near the end)
  timeline.to('.hero--container', { opacity: 1, xPercent: 0, duration: 0.8, ease: "power4.out" }, '-=0.5')
  timeline.to('.hero--scroller', { opacity: 1, y: '0%', duration: 0.6, ease: "power4.inOut" }, '-=0.6')

  // Update sidebar indicators
  timeline.to('.side-bar .choose', { opacity: 0.5, scale: 1, duration: 0.8, ease: "power4.inOut" }, '-=1.2')
  timeline.to('.side-bar .unique', { opacity: 1, scale: 1.5, duration: 0.8, ease: "power4.inOut" }, '-=0.8')
}

/**
 * Threshold-based wheel handler that triggers section transitions
 */
let lastWheelTime = 0;
const WHEEL_THROTTLE_MS = 32; // ~30fps, prevents excessive event processing

function onWheel(e) {
  // Throttle wheel events to prevent performance issues from rapid scrolling
  const now = Date.now();
  if (now - lastWheelTime < WHEEL_THROTTLE_MS) {
    return;
  }
  lastWheelTime = now;

  if (skipScrollAnimation) {
    return;
  }

  // Reset accumulated delta if animation is in progress to prevent stuttering
  if (isAnimating) {
    accumulatedDelta = 0;
    return;
  }

  // Accumulate delta until threshold is reached
  accumulatedDelta += e.deltaY;

  if (Math.abs(accumulatedDelta) >= SCROLL_THRESHOLD) {
    const direction = accumulatedDelta > 0 ? 1 : -1; // 1 = down, -1 = up

    if (direction > 0) {
      // Scrolling down
      if (currentSection === SECTION_HERO) {
        animateHeroToBrilliant();
      } else if (currentSection === SECTION_BRILLIANT) {
        animateBrilliantToChoose();
      }
    } else {
      // Scrolling up
      if (currentSection === SECTION_CHOOSE) {
        animateChooseToBrilliant();
      } else if (currentSection === SECTION_BRILLIANT) {
        animateBrilliantToHero();
      }
    }

    // Reset accumulated delta after triggering animation
    accumulatedDelta = 0;
  }
}


export function setRendererLoaded()
{
  isRendererLoaded = true;
}

export function getUiState() {
  return uiState;
}

export function isChanged() {
  return uiState.changed.length > 0;
}

export function clearChanged() {
  uiState.changed.length = 0;
}

// Updates UI selection highlight for gem or metal
// Called from Rust when switching rings to show the ring's saved colors
export function updateSelectionHighlight( type, value )
{
  // Validate value to prevent CSS selector injection
  if ( !/^[a-zA-Z0-9_-]+$/.test( value ) )
  {
    return;
  }

  if ( type === "gem" )
  {
    document.querySelector( '.colors--list li.active' )?.classList.remove( 'active' );
    document.querySelector( `.colors--list li.${value}` )?.classList.add( 'active' );
  }
  else if ( type === "metal" )
  {
    document.querySelector( '.materials--list li.active' )?.classList.remove( 'active' );
    document.querySelector( `.materials--list li.${value}` )?.classList.add( 'active' );
  }
}

export function enableDebugControls()
{
  const colorControls = document.querySelector( '.color-controls--container' )
  if ( colorControls )
  {
    colorControlsEnabled = true
    colorControls.style.display = 'flex';

    // Setup event listeners now that controls are visible
    setupColorPickerListeners();
  }
}

function setupColorPickerListeners() {
  // Helper function to convert hex color to RGB array
  function hexToRgb(hex) {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ?
      [
        parseInt(result[1], 16) / 255,
        parseInt(result[2], 16) / 255,
        parseInt(result[3], 16) / 255
      ] : [1.0, 1.0, 1.0];
  }

  // Gem color picker
  const gemColorPicker = document.getElementById('gem-color-picker');
  const gemMultiplier = document.getElementById('gem-multiplier');
  const gemMultiplierValue = document.getElementById('gem-multiplier-value');

  if (gemColorPicker && gemMultiplier && gemMultiplierValue) {
    gemColorPicker.addEventListener
      (
        'input',
        (e) => {
          const rgb = hexToRgb(e.target.value);
          uiState.gemCustomColor = rgb;
          uiState.gem = "custom";
          uiState.changed.push("gem");
        }
      );

    gemMultiplier.addEventListener
      (
        'input',
        (e) => {
          const value = parseFloat(e.target.value);
          // Validate parsed value to prevent NaN injection
          if (isNaN(value)) return;
          uiState.gemMultiplier = value;
          gemMultiplierValue.textContent = uiState.gemMultiplier.toFixed(1);
          uiState.changed.push("gem");
        }
      );
  }

  // Metal color picker
  const metalColorPicker = document.getElementById('metal-color-picker');
  const metalMultiplier = document.getElementById('metal-multiplier');
  const metalMultiplierValue = document.getElementById('metal-multiplier-value');

  if (metalColorPicker && metalMultiplier && metalMultiplierValue) {
    metalColorPicker.addEventListener
      (
        'input',
        (e) => {
          const rgb = hexToRgb(e.target.value);
          uiState.metalCustomColor = rgb;
          uiState.metal = "custom";
          uiState.changed.push("metal");
        }
      );

    metalMultiplier.addEventListener
      (
        'input',
        (e) => {
          const value = parseFloat(e.target.value);
          // Validate parsed value to prevent NaN injection
          if (isNaN(value)) return;
          uiState.metalMultiplier = value;
          metalMultiplierValue.textContent = uiState.metalMultiplier.toFixed(1);
          uiState.changed.push("metal");
        }
      );
  }
}

// -- MAIN PAGE --
async function replaceSVG(svgPath, selector) {
  let svg = document.querySelector(selector);
  if (!svg) return;

  const response = await fetch(svgPath);
  const svgText = await response.text();

  // Sanitize SVG content to prevent XSS attacks
  // DOMPurify.sanitize removes potentially malicious content like:
  // - Event handlers (onload, onclick, etc.)
  // - Embedded scripts
  // - Data URIs with JavaScript
  const cleanSvgText = DOMPurify.sanitize
  (
    svgText, 
    {
      USE_PROFILES : { svg: true },
      ADD_TAGS : [ 'use' ],  // Allow <use> tags for SVG references
      ADD_ATTR : [ 'xlink:href' ]  // Allow xlink:href for SVG links
    }
  );

  const parser = new DOMParser();
  const newSvgDoc = parser.parseFromString( cleanSvgText, 'image/svg+xml' );
  const newSvg = newSvgDoc.documentElement;

  if (svg.hasAttribute('width')) {
    newSvg.setAttribute('width', svg.getAttribute('width'));
  }
  if (svg.hasAttribute('height')) {
    newSvg.setAttribute('height', svg.getAttribute('height'));
  }

  svg.classList.forEach(cls => newSvg.classList.add(cls));

  try {
    svg.outerHTML = newSvg.outerHTML;
  }
  catch {

  }
}

function introAnimation() {
  firstLoad = false

  gsap.timeline({
    onComplete: () => {
      setupScrollAnimation();
      // Enable wheel navigation after intro animation
      skipScrollAnimation = false;
    }
  })
    .fromTo(uiState.position, { 0: 0.6373576, 1: 1.1441559, 2: -0.9127405 }, { 0: 0.6858612, 1: 2.7440538, 2: -0.026622068, duration: 4, onUpdate: () => { !uiState.changed.includes("position") && uiState.changed.push("position") } }, '-=0.8')
    .fromTo(uiState.target, { 0: 0.55595696, 1: 0.55741394, 2: -1.0331136 }, { 0: 0.36420232, 1: 0.8480059, 2: -0.36873266, duration: 4, onUpdate: () => { !uiState.changed.includes("target") && uiState.changed.push("target") } }, '-=4')
    .fromTo('.header--container', { opacity: 0, y: '-100%' }, { opacity: 1, y: '0%', ease: "power1.inOut", duration: 0.8 }) // , '-=1'
    .fromTo('.hero--scroller', { opacity: 0, y: '150%' }, { opacity: 1, y: '0%', ease: "power4.inOut", duration: 1 }, '-=1')
    .fromTo('.hero--container', { opacity: 0, x: '100%' }, { opacity: 1, x: '0%', ease: "power4.inOut", duration: 1.8 }, '-=1')
    .fromTo('.side-bar', { opacity: 0.0, x: '50%' }, { opacity: 1, x: '0%', ease: "power4.inOut", duration: 2 }, '-=1')
    // Initialize all sidebar circles with clearProps to remove inline styles
    .set('.side-bar .unique', { clearProps: "transform", opacity: 0.5, scale: 1 }, '-=2')
    .set('.side-bar .brilliant', { clearProps: "transform", opacity: 0.5, scale: 1 }, '-=2')
    .set('.side-bar .choose', { clearProps: "transform", opacity: 0.5, scale: 1 }, '-=2')
    // Animate only the active circle (unique for Hero section)
    .to('.side-bar .unique', { opacity: 1, scale: 1.5, ease: "power4.inOut", duration: 2 }, '-=2')
}

function enableScrollAnimationOnUserScroll() {
  window.addEventListener("wheel", onWheel, { passive: true });
}

/**
 * Configures the main page threshold-based scroll navigation system.
 *
 * This function sets up the wheel event handler that enables section-by-section
 * navigation without traditional scrolling. Instead of scroll position, wheel
 * events accumulate until a threshold is reached, triggering smooth GSAP animations
 * between sections.
 *
 * ## Animation Structure
 * Three main sections managed by wheel-based navigation:
 *
 * **Hero Section**
 * - Initial view showing hero content and scroll indicator
 * - Sidebar "unique" indicator highlighted
 * - Camera at position [0.6858612, 2.7440538, -0.026622068]
 *
 * **Brilliant Section**
 * - Brilliant content visible with text background
 * - Sidebar "brilliant" indicator active
 * - Camera at position [-0.40259048, 2.6242757, -0.18104002]
 *
 * **Choose Section**
 * - Choose content with ring selection interface
 * - Sidebar "choose" indicator active
 * - Camera at position [-0.39456308, 2.431139, 0.23367776]
 *
 * ## Navigation Control
 * - Wheel events accumulate deltaY until SCROLL_THRESHOLD is reached
 * - Once threshold exceeded, appropriate section transition animation plays
 * - isAnimating flag blocks further navigation during transitions
 * - Section transitions use GSAP timelines
 *
 * @see animateHeroToBrilliant/animateBrilliantToHero/animateBrilliantToChoose/animateChooseToBrilliant - Section transition animations
 * @see onWheel - Threshold detection and direction handling
 */
function setupScrollAnimation() {
  enableScrollAnimationOnUserScroll();
  // Prevent browser's default scroll behavior
  document.body.style.overflow = "hidden"
}

function onCompleteConfigAnimation() {
  setTimeout
  (
    () =>
    {
      uiState.transitionAnimationEnabled = false;
    },
    100
  );

  const canvas = document.querySelector('.canvas')
  const colorControls = document.querySelector('.color-controls--container')

  camView3.style.display = "none"

  canvas.style.pointerEvents = "all";
  uiState.state = "configurator";
  uiState.changed.push("state");

  if ( colorControls && colorControlsEnabled ) 
  {
    colorControls.style.display = "flex"
    colorControls.style.pointerEvents = "all";
  }
}

/**
 * Animates the transition from main page to configuration mode.
 *
 * This function orchestrates the visual transition when the user clicks "Choose Your Ring"
 * to enter the configuration interface. It plays a carefully choreographed camera movement
 * and UI transition sequence.
 *
 * ## Animation Sequence (3.75 seconds total)
 *
 * **Camera Movement** (parallel, both 3.75s duration)
 * - Position: Moves from Choose section view to overhead configurator view
 *   - From: (-0.39456308, 2.431139, 0.23367776)
 *   - To: (-1.2621417, 4.005461, 1.2621417)
 * - Target: Refocuses camera from angled view to centered top-down view
 *   - From: (0.2921338, 0.9732934, -0.18001612)
 *   - To: (0, 0.6, 0)
 *
 * **UI Transitions** (parallel with camera)
 * - Choose content slides out right (x: 0% → 200%, opacity: 1 → 0)
 * - Choose background text slides out right
 * - Footer menu slides up from bottom (y: 150% → 0%, opacity: 0 → 1)
 *   Delayed by 2.25s (starts at -=1.5 mark)
 *
 * ## State Management
 *
 * **During Animation**
 * - `uiState.transitionAnimationEnabled = true` - Signals active transition
 * - `isAnimating = true` - Blocks wheel navigation
 * - Camera view section markers hidden (cam-view-1/2/3)
 * - Camera position/target updates pushed to `uiState.changed` array
 *
 * **After Animation (onCompleteConfigAnimation)**
 * - `uiState.transitionAnimationEnabled = false` (after 100ms delay)
 * - `uiState.state = "configurator"` - Activates configurator mode
 * - Configuration UI elements made visible (footer menu, gem/material/ring selectors)
 * - Canvas pointer events enabled for ring interaction
 *
 * @see onCompleteConfigAnimation - Callback that finalizes the transition
 * @see exitConfigAnimation - Reverse animation returning to main page
 */
function configAnimation() {
  uiState.transitionAnimationEnabled = true;
  isAnimating = true;

  camView1.style.display = "none"
  camView2.style.display = "none"

  exitContainer.style.display = "flex"
  exitContainer.style.pointerEvents = "all";
  gemMenu.style.display = "flex"
  footerMenu.style.display = "flex"
  materialsMenu.style.display = "flex"
  ringsMenu.style.display = "flex"
  configMaterial.style.display = "flex"
  configGem.style.display = "flex"
  closeConfigMaterial.style.display = "flex"
  configRing.style.display = "flex"
  closeConfigGem.style.display = "flex"
  closeConfigRing.style.display = "flex"
  footerContainer.style.display = "flex"

  gsap.timeline({ onComplete: onCompleteConfigAnimation })
    // Set initial hidden state for footer menu and exit container
    .set('.footer--menu', { opacity: 0, y: '150%' })
    .set('.exit--container', { opacity: 0, y: '-150%' })
    .fromTo
    (
      uiState.position,
      { 0: -0.39456308, 1: 2.431139, 2: 0.23367776 },
      {
        0: -1.2621417, 1: 4.005461, 2: 1.2621417,
        duration: 3.75,
        onUpdate: () => { !uiState.changed.includes("position") && uiState.changed.push("position") }
      }
    )
    .fromTo
    (
      uiState.target,
      { 0: 0.2921338, 1: 0.9732934, 2: -0.18001612 },
      {
        0: 0, 1: 0.6, 2: 0, duration: 3.75,
        onUpdate: () => { !uiState.changed.includes("target") && uiState.changed.push("target") }
      },
      '-=3.75'
    )
    .fromTo('.choose--content', { opacity: 1, x: '0%', y: '0%' }, { opacity: 0, y: '-100%', duration: 2.25, ease: "power4.out" }, '-=3.75')
    .fromTo('.choose--text-bg', { opacity: 0.1, x: '0%', y: '0%' }, { opacity: 0, y: '-100%', duration: 2.25, ease: "power4.out" }, '-=3.75')
    .to('.footer--menu', { opacity: 1, y: '0%', duration: 2.25, ease: "power4.out" }, '-=1.5')
    .to('.exit--container', { opacity: 1, y: '0%', duration: 2.25, ease: "power4.out" }, '-=2.25')
}

/**
 * Animates the transition from configuration mode back to main page.
 *
 * Returns to Hero section with camera animation and UI restoration.
 * Resets section state and re-enables wheel navigation.
 */
function exitConfigAnimation() {
  gemMenu.classList.remove('show')
  materialsMenu.classList.remove('show')
  ringsMenu.classList.remove('show')
  if (document.querySelector('.footer--menu li.active')) {
    document.querySelector('.footer--menu li.active')?.classList.remove('active')
  }

  isAnimating = true;

  // Hide exit container and footer menu immediately
  exitContainer.style.display = "none"
  exitContainer.style.pointerEvents = "none"
  footerContainer.style.display = "none"

  gsap.timeline({
    onComplete: () => {
      // Reset to Hero section and enable navigation
      currentSection = SECTION_HERO;
      isAnimating = false;
      accumulatedDelta = 0;
      skipScrollAnimation = false;
      // Re-attach wheel event listener
      window.addEventListener("wheel", onWheel, { passive: true });
    }
  })
    .to(uiState.position, { 0: 0.6858612, 1: 2.7440538, 2: -0.026622068, duration: 1.2, ease: "power4.out", onUpdate: () => { !uiState.changed.includes("position") && uiState.changed.push("position") } })
    .to(uiState.target, {
      0: 0.36420232, 1: 0.8480059, 2: -0.36873266, duration: 1.2, ease: "power4.out",
      onUpdate: () => { !uiState.changed.includes("target") && uiState.changed.push("target") }
    },
      '-=1.2'
    )
    // Reset choose content position to initial state
    .set('.choose--content', { opacity: 0, x: '0%', y: '0%' })
    .set('.choose--text-bg', { opacity: 0, x: '0%', y: '0%' })
    // Show hero content simultaneously with camera movement
    .to('.hero--container', { opacity: 1, xPercent: 0, duration: 1.2, ease: "power4.out" }, '-=1.2')
    .to('.hero--scroller', { opacity: 1, y: '0%', duration: 0.8, ease: "power4.inOut" }, '-=0.8')
    // Update sidebar to Hero section - reset all dots and activate Hero
    .to('.side-bar .brilliant', { opacity: 0.5, scale: 1, duration: 0.8, ease: "power4.inOut" }, '-=1.2')
    .to('.side-bar .choose', { opacity: 0.5, scale: 1, duration: 0.8, ease: "power4.inOut" }, '-=1.2')
    .to('.side-bar .unique', { opacity: 1, scale: 1.5, duration: 0.8, ease: "power4.inOut" }, '-=0.8')
}

function setupNavigationListeners() {
  // Navigate to Brilliant section via scroll down button
  document.querySelector('.button-scroll')?.addEventListener
    (
      'click',
      () => {
        if (currentSection === SECTION_HERO && !isAnimating && !skipScrollAnimation) {
          animateHeroToBrilliant();
        }
      }
    )

  // Navigate to Hero section via sidebar (from any section)
  document.querySelector('.unique')?.addEventListener
    (
      'click',
      () => {
        if ( skipScrollAnimation || isAnimating ) {
          return;
        }

        if ( currentSection === SECTION_BRILLIANT )
        {
          animateBrilliantToHero();
        }
        else if ( currentSection === SECTION_CHOOSE )
        {
          animateChooseToHero();
        }
      }
    )

  // Navigate to Brilliant section via sidebar (from any section)
  document.querySelector( '.brilliant' )?.addEventListener
    (
      'click',
      () =>
      {
        if ( skipScrollAnimation || isAnimating ) {
          return;
        }

        if (currentSection === SECTION_HERO )
        {
          animateHeroToBrilliant();
        }
        else if ( currentSection === SECTION_CHOOSE )
        {
          animateChooseToBrilliant();
        }
      }
    )

  // Navigate to Choose section via sidebar (from any section)
  document.querySelector( '.choose' )?.addEventListener
    (
      'click',
      () =>
      {
        if ( skipScrollAnimation || isAnimating ) {
          return;
        }

        if ( currentSection === SECTION_HERO )
        {
          animateHeroToChoose();
        }
        else if ( currentSection === SECTION_BRILLIANT )
        {
          animateBrilliantToChoose();
        }
      }
    )

  // Navigate to Brilliant section via hero scroller
  document.querySelector( '.hero--scroller' )?.addEventListener
    (
      'click',
      () =>
      {
        if ( currentSection === SECTION_HERO && !isAnimating && !skipScrollAnimation )
        {
          animateHeroToBrilliant();
        }
      }
    )
}

document.querySelector('.btn-customize')?.addEventListener
  (
    'click',
    () => {
      skipScrollAnimation = true;
      window.removeEventListener("wheel", onWheel, { passive: true });

      uiState.state = "configurator";
      uiState.changed.push("state");
      exploreView.style.pointerEvents = "none"
      document.body.style.overflow = "hidden"
      document.body.style.cursor = "grab"
      sidebar.style.display = "none"
      headerContainer.style.display = "none"

      configAnimation()
    }
  )

document.querySelector('.button--exit')?.addEventListener
  (
    'click',
    () => {
      const canvas = document.querySelector('.canvas')
      const colorControls = document.querySelector('.color-controls--container')

      // Reset all ring states to defaults
      uiState.gem = "white"
      uiState.metal = "silver"
      uiState.ring = 0
      uiState.changed.push("gem")
      uiState.changed.push("metal")
      uiState.changed.push("ring")
      uiState.changed.push("reset")

      // Reset active elements to default values
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.colors--list li.white')?.classList.add('active')
      document.querySelector('.materials--list li.active')?.classList.remove('active')
      document.querySelector('.materials--list li.silver')?.classList.add('active')
      document.querySelector('.rings--list li.active')?.classList.remove('active')
      document.querySelector('.rings--list li.ring0')?.classList.add('active')

      camView1.style.display = "flex"
      camView2.style.display = "flex"
      camView3.style.display = "flex"
      headerContainer.style.display = "flex"

      canvas.style.pointerEvents = "none";
      uiState.state = "hero";
      uiState.changed.push("state");

      if (colorControls && colorControlsEnabled) {
        colorControls.style.display = "none"
        colorControls.style.pointerEvents = "none";
      }
      // exitContainer and footerContainer will be hidden after animation completes
      gemMenu.style.display = "none"
      materialsMenu.style.display = "none"
      ringsMenu.style.display = "none"
      configMaterial.style.display = "none"
      configGem.style.display = "none"
      closeConfigMaterial.style.display = "none"
      configRing.style.display = "none"
      closeConfigGem.style.display = "none"
      closeConfigRing.style.display = "none"

      exploreView.style.pointerEvents = "all"
      document.body.style.overflow = "hidden"
      document.body.style.cursor = "auto"
      sidebar.style.display = "block"
      exitConfigAnimation()
    }
  )

async function setupMainPage() {
  replaceSVG("./static/images/jewelry_site/icons/gem.svg", ".image--gem")
  replaceSVG("./static/images/jewelry_site/icons/metal.svg", ".image--material")
  replaceSVG("./static/images/jewelry_site/icons/ring.svg", ".image--ring")

  window.addEventListener
    (
      "load",
      () => {
        const start = () => {
          if (isRendererLoaded) {
            if (firstLoad) {
              if (uiState.state == "hero") {
                introAnimation()
              }
            }
            else {
              setupScrollAnimation()
            }
          }
          else {
            requestAnimationFrame(start);
          }
        };
        start();
      }
    )
}

// -- CONFIGURATOR --
function setActive(target, groupSelector) {
  document.querySelectorAll(`${groupSelector} li.active`)
    .forEach
    (
      li => li.classList.remove("active")
    );
  target.classList.add("active");
}

function bindSelector(groupSelector, type, valueGetter) {
  document.querySelectorAll(`${groupSelector} li`).forEach
    (
      li => {
        li.addEventListener
          (
            "click",
            () => {
              setActive(li, groupSelector);
              uiState[type] = valueGetter(li);
              uiState.changed.push(type);
            }
          );
      }
    );
}

// GEM MENU
configGem.addEventListener
  (
    'click',
    () => {
      gemMenu.classList.add('show')
      materialsMenu.classList.remove('show')
      ringsMenu.classList.remove('show')

      gemMenu.style.zIndex = 1
      materialsMenu.style.zIndex = 0
      ringsMenu.style.zIndex = 0

      if (document.querySelector('.footer--menu li.active')) {
        document.querySelector('.footer--menu li.active')?.classList.remove('active')
      }
      configGem.parentElement?.classList.add('active')
    }
  )

// DIAMOND COLORS
document.querySelector('.ruby')?.addEventListener
  (
    'click',
    () => {
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.ruby')?.classList.add('active')
    }
  )
document.querySelector('.white')?.addEventListener
  (
    'click',
    () => {
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.white')?.classList.add('active')
    }
  )
document.querySelector('.emerald')?.addEventListener
  (
    'click',
    () => {
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.emerald')?.classList.add('active')
    }
  )

// MATERIALS MENU
configMaterial.addEventListener
  (
    'click',
    () => {
      materialsMenu.classList.add('show')
      gemMenu.classList.remove('show')
      ringsMenu.classList.remove('show')

      materialsMenu.style.zIndex = 1
      gemMenu.style.zIndex = 0
      ringsMenu.style.zIndex = 0

      if (document.querySelector('.footer--menu li.active')) {
        document.querySelector('.footer--menu li.active')?.classList.remove('active')
      }
      configMaterial.parentElement?.classList.add('active')
    }
  )

// DIAMOND COLORS
document.querySelector('.red')?.addEventListener
  (
    'click',
    () => {
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.red')?.classList.add('active')
    }
  )

document.querySelector('.green')?.addEventListener
  (
    'click',
    () => {
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.green')?.classList.add('active')
    }
  )

document.querySelector('.turquoise')?.addEventListener
  (
    'click',
    () => {
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.turquoise')?.classList.add('active')
    }
  )

document.querySelector('.yellow')?.addEventListener
  (
    'click',
    () => {
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.yellow')?.classList.add('active')
    }
  )

document.querySelector('.blue')?.addEventListener
  (
    'click',
    () => {
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.blue')?.classList.add('active')
    }
  )

document.querySelector('.orange')?.addEventListener
  (
    'click',
    () => {
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.orange')?.classList.add('active')
    }
  )

document.querySelector('.pink')?.addEventListener
  (
    'click',
    () => {
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.pink')?.classList.add('active')
    }
  )

document.querySelector('.white')?.addEventListener
  (
    'click',
    () => {
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.white')?.classList.add('active')
    }
  )

// MATERIALS COLOR
document.querySelector('.silver')?.addEventListener
  (
    'click',
    () => {
      document.querySelector('.materials--list li.active')?.classList.remove('active')
      document.querySelector('.silver')?.classList.add('active')
    }
  )
document.querySelector('.copper')?.addEventListener
  (
    'click',
    () => {
      document.querySelector('.materials--list li.active')?.classList.remove('active')
      document.querySelector('.copper')?.classList.add('active')
    }
  )
document.querySelector('.gold')?.addEventListener
  (
    'click',
    () => {
      document.querySelector('.materials--list li.active')?.classList.remove('active')
      document.querySelector('.gold')?.classList.add('active')
    }
  )

// CHANGE RING
configRing.addEventListener
  (
    'click',
    () => {
      ringsMenu.classList.add('show')
      materialsMenu.classList.remove('show')
      gemMenu.classList.remove('show')

      ringsMenu.style.zIndex = 1
      materialsMenu.style.zIndex = 0
      gemMenu.style.zIndex = 0

      if (document.querySelector('.footer--menu li.active')) {
        document.querySelector('.footer--menu li.active')?.classList.remove('active')
      }
      configRing.parentElement?.classList.add('active')
    }
  )

// CLOSE GEM MENU
closeConfigGem.addEventListener
  (
    'click',
    () => {
      gemMenu.classList.remove('show')

      if (document.querySelector('.footer--menu li.active')) {
        document.querySelector('.footer--menu li.active')?.classList.remove('active')
      }
    }
  )

// CLOSE MATERIAL MENU
closeConfigMaterial.addEventListener
  (
    'click',
    () => {
      materialsMenu.classList.remove('show')

      if (document.querySelector('.footer--menu li.active')) {
        document.querySelector('.footer--menu li.active')?.classList.remove('active')
      }
    }
  )

// CLOSE RING MENU
closeConfigRing.addEventListener
  (
    'click',
    () => {
      ringsMenu.classList.remove('show')

      if (document.querySelector('.footer--menu li.active')) {
        document.querySelector('.footer--menu li.active')?.classList.remove('active')
      }
    }
  )

function setupConfigurator() {
  bindSelector
    (
      ".colors--list",
      "gem",
      li => {
        if (li.classList.contains("white")) return "white";
        if (li.classList.contains("red")) return "red";
        if (li.classList.contains("orange")) return "orange";
        if (li.classList.contains("yellow")) return "yellow";
        if (li.classList.contains("green")) return "green";
        if (li.classList.contains("turquoise")) return "turquoise";
        if (li.classList.contains("blue")) return "blue";
        if (li.classList.contains("pink")) return "pink";
        return uiState.gem;
      }
    );

  bindSelector
    (
      ".materials--list",
      "metal",
      li => {
        if (li.classList.contains("silver")) return "silver";
        if (li.classList.contains("gold")) return "gold";
        if (li.classList.contains("copper")) return "copper";
        return uiState.metal;
      }
    );

  bindSelector
    (
      ".rings--list",
      "ring",
      li => {
        if (li.classList.contains("ring0")) return 0;
        if (li.classList.contains("ring1")) return 1;
        if (li.classList.contains("ring2")) return 2;

        return uiState.ring;
      }
    );

  exitContainer.style.display = "none"
  gemMenu.style.display = "none"
  footerMenu.style.display = "none"
  materialsMenu.style.display = "none"
  ringsMenu.style.display = "none"
  configMaterial.style.display = "none"
  configGem.style.display = "none"
  closeConfigMaterial.style.display = "none"
  configRing.style.display = "none"
  closeConfigGem.style.display = "none"
  closeConfigRing.style.display = "none"
  footerContainer.style.display = "none"
}

document.addEventListener
(
  "DOMContentLoaded",
  () => {
    setupMainPage();
    setupConfigurator();
    setupNavigationListeners();
  }
);

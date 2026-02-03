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

let currentScroll = window.scrollY
let isTicking = false
let scrolled = false
let isRendererLoaded = false
let resized = false

function getMaxScroll() {
  return document.documentElement.scrollHeight - window.innerHeight
}

// Firefox uses async scroll, so we need to handle it differently
const isFirefox = navigator.userAgent.includes("Firefox") &&
  !navigator.userAgent.includes("Seamonkey");

function onWheel(e) {
  if (scrolled) {
    scrolled = false
    return
  }

  currentScroll = window.scrollY + e.deltaY
  currentScroll = Math.max(0, Math.min(currentScroll, getMaxScroll()))
  resized = false

  if (!isTicking) {
    isTicking = true

    requestAnimationFrame
      (
        () => {
          isTicking = false
          resized = false

          if (skipScrollAnimation){
            return
          }

          window.scrollTo(0, currentScroll)
        }
      )
  }
}

if (!isFirefox) {
  window.addEventListener("scroll", () => {
    scrolled = true
  })
}

window.addEventListener("resize", () => {
  if (window.scrollY == 0){
    resized = true
  }
})

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
          uiState.gemMultiplier = parseFloat(e.target.value);
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
          uiState.metalMultiplier = parseFloat(e.target.value);
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

  const parser = new DOMParser();
  const newSvgDoc = parser.parseFromString(svgText, 'image/svg+xml');
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

  gsap.timeline()
    .fromTo(uiState.position, { 0: 0.6373576, 1: 1.1441559, 2: -0.9127405 }, { 0: 0.6858612, 1: 2.7440538, 2: -0.026622068, duration: 4, onUpdate: () => { !uiState.changed.includes("position") && uiState.changed.push("position") } }, '-=0.8')
    .fromTo(uiState.target, { 0: 0.55595696, 1: 0.55741394, 2: -1.0331136 }, { 0: 0.36420232, 1: 0.8480059, 2: -0.36873266, duration: 4, onUpdate: () => { !uiState.changed.includes("target") && uiState.changed.push("target") } }, '-=4')
    .fromTo('.header--container', { opacity: 0, y: '-100%' }, { opacity: 1, y: '0%', ease: "power1.inOut", duration: 0.8 }) // , '-=1'
    .fromTo('.hero--scroller', { opacity: 0, y: '150%' }, { opacity: 1, y: '0%', ease: "power4.inOut", duration: 1 }, '-=1')
    .fromTo('.hero--container', { opacity: 0, x: '100%' }, { opacity: 1, x: '0%', ease: "power4.inOut", duration: 1.8, onComplete: setupScrollAnimation }, '-=1')
    .fromTo('.side-bar', { opacity: 0.0, x: '50%' }, { opacity: 1, x: '0%', ease: "power4.inOut", duration: 2 }, '-=1')
    .to('.side-bar .unique', { opacity: 1, scale: 1.5, ease: "power4.inOut", duration: 2 }, '-=1')
}

function enableScrollAnimationOnUserScroll() {
  const onScroll = () => {
    skipScrollAnimation = false;
    window.removeEventListener("scroll", onScroll);
  };

  if (!isFirefox) {
    window.addEventListener("wheel", onWheel, { passive: true });
  }
  window.addEventListener("scroll", onScroll, { passive: true });
}

function setupScrollAnimation() {
  enableScrollAnimationOnUserScroll();
  document.body.style.overflowY = "scroll"

  const scrollAnimation = gsap.timeline({ default: { ease: 'none' } })
  window.scrollAnimation = scrollAnimation;

  // BRILLIANT
  scrollAnimation
    .to
    (
      '.hero--scroller',
      {
        opacity: 0,
        y: '150%',
        scrollTrigger: { trigger: ".cam-view-2", start: "top bottom", end: "top center", scrub: 1, scrub: true, invalidateOnRefresh: true, immediateRender: false }
      }
    )
    .to
    (
      '.hero--container',
      {
        opacity: 0, xPercent: '100', ease: "power4.out", scrollTrigger: { trigger: ".cam-view-2", start: "top bottom", end: "top top", scrub: true, invalidateOnRefresh: true, immediateRender: false }
      }
    )
    .to
    (
      '.brilliant--text-bg',
      {
        opacity: 0.1, ease: "power4.inOut", scrollTrigger: { trigger: ".cam-view-2", start: "top bottom", end: 'top top', scrub: true, invalidateOnRefresh: true, immediateRender: false }
      }
    )
    .fromTo
    (
      '.brilliant--container',
      { opacity: 0, x: '-110%' },
      { opacity: 1, x: '0%', ease: "power4.inOut", scrollTrigger: { trigger: ".cam-view-2", start: "top bottom", end: 'top top', scrub: true, invalidateOnRefresh: true, immediateRender: false } }
    )
    .fromTo
    (
      uiState.position,
      { 0: 0.6858612, 1: 2.7440538, 2: -0.026622068 },
      {
        0: -0.40259048, 1: 2.6242757, 2: -0.18104002,
        scrollTrigger: { trigger: ".cam-view-2", start: "top bottom", end: "top top", scrub: true, invalidateOnRefresh: true, immediateRender: false },
        onUpdate: () => {
          !resized &&
          (() => { resized = false; return true; })() &&
          !skipScrollAnimation && !uiState.changed.includes("position") && uiState.changed.push("position")
        }
      }
    )
    .fromTo
    (
      uiState.target,
      { 0: 0.36420232, 1: 0.8480059, 2: -0.36873266 },
      {
        0: -0.23794234, 1: 0.49070162, 2: -0.32702705,
        scrollTrigger: { trigger: ".cam-view-2", start: "top bottom", end: "top top", scrub: true, invalidateOnRefresh: true, immediateRender: false },
        onUpdate: () => {
          !resized &&
          (() => { resized = false; return true; })() &&
          !skipScrollAnimation && !uiState.changed.includes("target") && uiState.changed.push("target")
        }
      }
    )
    .addLabel("Brilliant")
    .to('.side-bar .unique', { opacity: 1, scale: 1.5, ease: "power4.inOut", duration: 2, scrollTrigger: { trigger: ".cam-view-1", start: "top bottom", end: 'top top', scrub: true, invalidateOnRefresh: true, immediateRender: false } })
    .to('.side-bar .unique', { opacity: 0.5, scale: 1, ease: "power4.inOut", duration: 2, scrollTrigger: { trigger: ".cam-view-2", start: "top bottom", end: 'top top', scrub: true, invalidateOnRefresh: true, immediateRender: false } })
    .to('.side-bar .brilliant', { opacity: 1, scale: 1.5, ease: "power4.inOut", duration: 2, scrollTrigger: { trigger: ".cam-view-2", start: "top bottom", end: 'top top', scrub: true, invalidateOnRefresh: true, immediateRender: false } })

    // CHOOSE SECTION
    .to
    (
      '.brilliant--container',
      {
        opacity: 0, x: '-110%', ease: "power4.inOut",
        scrollTrigger: { trigger: ".cam-view-3", start: "top bottom", end: 'top top', scrub: true, invalidateOnRefresh: true, immediateRender: false }
      }
    )
    .to
    (
      '.choose--text-bg',
      {
        opacity: 0.1, x: '0%', ease: "power4.inOut",
        scrollTrigger: { trigger: ".cam-view-3", start: "top bottom", end: 'top top', scrub: true, invalidateOnRefresh: true, immediateRender: false }
      }
    )
    .fromTo
    (
      '.choose--content',
      { opacity: 0, x: '200%', y: '130%' },
      {
        opacity: 1, x: '0%', y: '0%', duration: 0.5, ease: "power4.inOut",
        scrollTrigger: { trigger: ".cam-view-3", start: "top bottom", end: "top top", scrub: true, invalidateOnRefresh: true, immediateRender: false }
      }
    )
    .fromTo
    (
      uiState.position,
      { 0: -0.40259048, 1: 2.6242757, 2: -0.18104002 },
      {
        0: -0.39456308, 1: 2.431139, 2: 0.23367776,
        scrollTrigger: { trigger: ".cam-view-3", start: "top bottom", end: "top top", scrub: true, invalidateOnRefresh: true, immediateRender: false },
        onUpdate: () => {
          !resized &&
          (() => { resized = false; return true; })() &&
          !skipScrollAnimation && !uiState.changed.includes("position") && uiState.changed.push("position")
        }
      }
    )
    .fromTo
    (
      uiState.target,
      { 0: -0.23794234, 1: 0.49070162, 2: -0.32702705 },
      {
        0: 0.2921338, 1: 0.9732934, 2: -0.18001612,
        scrollTrigger: { trigger: ".cam-view-3", start: "top bottom", end: "top top", scrub: true, invalidateOnRefresh: true, immediateRender: false },
        onUpdate: () => {
          !resized &&
          (() => { resized = false; return true; })() &&
          !skipScrollAnimation && !uiState.changed.includes("target") && uiState.changed.push("target")
        }
      }
    )
    .addLabel("Choose")
    .to
    (
      '.side-bar .brilliant',
      {
        opacity: 0.5, scale: 1, ease: "power4.inOut", duration: 2,
        scrollTrigger: { trigger: ".cam-view-3", start: "top bottom", end: 'top top', scrub: true, invalidateOnRefresh: true, immediateRender: false }
      }
    )
    .to
    (
      '.side-bar .choose',
      {
        opacity: 1, scale: 1.5, ease: "power4.inOut", duration: 2,
        scrollTrigger: { trigger: ".cam-view-3", start: "top bottom", end: 'top top', scrub: true, invalidateOnRefresh: true, immediateRender: false }
      }
    )
    .to
    (
      '.side-bar .brilliant',
      {
        opacity: 0.5, scale: 1, ease: "power4.inOut", duration: 2,
        scrollTrigger: { trigger: ".cam-view-1", start: "top bottom", end: 'top top', scrub: true, invalidateOnRefresh: true, immediateRender: false }
      }
    )
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

  canvas.style.pointerEvents = "all";
  uiState.state = "configurator";
  uiState.changed.push("state");

  if (colorControls && colorControlsEnabled) {
    colorControls.style.display = "flex"
    colorControls.style.pointerEvents = "all";
  }
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
}

function configAnimation() {
  uiState.transitionAnimationEnabled = true;

  // Disable all ScrollTrigger animations to prevent conflict with config entry animation
  ScrollTrigger.getAll().forEach(st => st.disable())

  camView1.style.display = "none"
  camView2.style.display = "none"
  camView3.style.display = "none"

  gsap.timeline()
    .fromTo
    (
      uiState.position,
      { 0: -0.39456308, 1: 2.431139, 2: 0.23367776 },
      {
        0: -1.2621417, 1: 4.005461, 2: 1.2621417,
        duration: 2.5,
        onUpdate: () => { !uiState.changed.includes("position") && uiState.changed.push("position") }
      }
    )
    .fromTo
    (
      uiState.target,
      { 0: 0.2921338, 1: 0.9732934, 2: -0.18001612 },
      {
        0: 0, 1: 0.6, 2: 0, duration: 2.5,
        onUpdate: () => { !uiState.changed.includes("target") && uiState.changed.push("target") }
      },
      '-=2.5'
    )
    .to('.choose--content', { opacity: 0, x: '200%', duration: 1.5, ease: "power4.out", onComplete: onCompleteConfigAnimation }, '-=2.5')
    .to('.choose--text-bg', { opacity: 0, x: '200%', duration: 1.5, ease: "power4.out" }, '-=2.5')
    .fromTo('.footer--menu', { opacity: 0, y: '150%' }, { opacity: 1, y: '0%', duration: 1.5 }, '-=1.0')
}

// EXIT EVENT
function exitConfigAnimation() {
  gemMenu.classList.remove('show')
  materialsMenu.classList.remove('show')
  ringsMenu.classList.remove('show')
  if (document.querySelector('.footer--menu li.active')) {
    document.querySelector('.footer--menu li.active')?.classList.remove('active')
  }
  document.body.style.overflowY = "hidden"

  // Disable all ScrollTrigger animations to prevent conflict with exit animation
  ScrollTrigger.getAll().forEach(st => st.disable())

  gsap.timeline()
    .to(uiState.position, { 0: 0.6858612, 1: 2.7440538, 2: -0.026622068, duration: 1.2, ease: "power4.out", onUpdate: () => { !uiState.changed.includes("position") && uiState.changed.push("position") } })
    .to(uiState.target, {
      0: 0.36420232, 1: 0.8480059, 2: -0.36873266, duration: 1.2, ease: "power4.out",
      onUpdate: () => { window.scrollTo(0, 0); !uiState.changed.includes("target") && uiState.changed.push("target") },
      onComplete: () => {
        // Re-enable ScrollTrigger animations after exit animation completes
        ScrollTrigger.getAll().forEach(st => st.enable())

        if (!isFirefox) {
          window.addEventListener("wheel", onWheel, { passive: true });
        }
        skipScrollAnimation = false
        document.body.style.overflowY = "scroll"
      }
    },
      '-=1.2'
    )
    .to('.footer--menu', { opacity: 0, y: '150%' })
    .to('.choose--content', { opacity: 1, x: '0%', duration: 0.5, ease: "power4.out" }, '-=1.2')
}

document.querySelector('.button-scroll')?.addEventListener
  (
    'click',
    () => {
      const element = document.querySelector('.cam-view-2');
      if (element) {
        const top = element.getBoundingClientRect().top + window.scrollY;
        window.scrollTo({ top, left: 0, behavior: 'smooth' });
        skipScrollAnimation = false
      }
    }
  )

document.querySelector('.brilliant')?.addEventListener
  (
    'click',
    () => {
      const element = document.querySelector('.cam-view-2')
      window.scrollTo({ top: element?.getBoundingClientRect().top, left: 0, behavior: 'smooth' })
    }
  )

document.querySelector('.hero--scroller')?.addEventListener
  (
    'click',
    () => {
      const element = document.querySelector('.cam-view-2')
      window.scrollTo({ top: element?.getBoundingClientRect().top, left: 0, behavior: 'smooth' })
    }
  )

document.querySelector('.btn-customize')?.addEventListener
  (
    'click',
    () => {
      skipScrollAnimation = true
      if (!isFirefox) {
        window.removeEventListener("wheel", onWheel, { passive: true })
      }

      uiState.state = "configurator";
      uiState.changed.push("state");
      exploreView.style.pointerEvents = "none"
      document.body.style.overflowY = "hidden"
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

      uiState.gem = "white"
      uiState.metal = "silver"
      uiState.ring = 0
      uiState.changed.push("gem")
      uiState.changed.push("metal")
      uiState.changed.push("ring")

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
      exitContainer.style.display = "none"
      exitContainer.style.pointerEvents = "none";
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

      exploreView.style.pointerEvents = "all"
      document.body.style.overflowY = "auto"
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
      (ev) => {
        const start = () => {
          if (isRendererLoaded) {
            window.scrollTo(0, 0)

            if (firstLoad && scrollY == 0) {
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
        if (li.classList.contains("ring3")) return 3;
        if (li.classList.contains("ring4")) return 4;

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
    ScrollTrigger.normalizeScroll(true);
    setupMainPage();
    setupConfigurator();
  }
);

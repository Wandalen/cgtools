let firstLoad = true

// -- MAIN PAGE --

async function setupMainPage()
{
  const position = { x : 0.0, y : 0.0, z : 0.0 }
  const target = { x : 0.0, y : 0.0, z : 0.0 }

  const exploreView = document.querySelector('.cam-view-3')
  const sidebar = document.querySelector('.side-bar')
  const headerContainer = document.querySelector('.header--container')
  const camView1 = document.querySelector('.cam-view-1')
  const camView2 = document.querySelector('.cam-view-2')
  const camView3 = document.querySelector('.cam-view-3')
  const emotionsImage = document.querySelector('.emotions--image')

  const previewContainer = document.querySelector(".preview--image");
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

  window.addEventListener
  (
    "load",
    ( ev ) =>
    {
      if ( firstLoad )
      {
        introAnimation()
      }
    }
  )

  window.scrollTo(0,0)

  function introAnimation()
  {
    firstLoad = false
    const introTL = gsap.timeline()
    introTL
    .fromTo( position, { x : 3, y : -0.8, z : 1.2 }, { x : 1.28, y : -1.7, z : 5.86, duration : 4 }, '-=0.8' )
    .fromTo( target, { x : 2.5, y : -0.07, z : -0.1 }, { x : 0.91, y: 0.03, z : -0.25, duration : 4 }, '-=4' )
    .fromTo( '.header--container', { opacity : 0, y : '-100%' }, {opacity : 1, y : '0%', ease : "power1.inOut", duration : 0.8 }, '-=1' )
    .fromTo( '.hero--image', { opacity : 0, x : '-200%' }, {opacity : 1, x : '-72%', ease : "power1.inOut", duration : 1.8 }, '-=1' )
    .fromTo( '.hero--scroller', { opacity : 0, y : '150%' }, { opacity : 1, y : '0%', ease : "power4.inOut", duration : 1 }, '-=1' )
    .fromTo( '.hero--container', { opacity : 0, x : '100%' }, { opacity : 1, x : '0%', ease : "power4.inOut", duration : 1.8, onComplete : setupScrollAnimation }, '-=1' )
    .fromTo( '.side-bar', { opacity : 0.0, x : '50%' }, { opacity : 1, x : '0%', ease : "power4.inOut", duration : 2 }, '-=1' )
    .to( '.side-bar .unique', { opacity : 1, scale : 1.5, ease : "power4.inOut", duration : 2 }, '-=1' )
  }

  function setupScrollAnimation()
  {
    document.body.style.overflowY = "scroll"

    const tl = gsap.timeline( { default : { ease: 'none' } } )

    // FOREVER
    tl
    .to
    (
      position, { x : -1.83, y : -0.14, z : 6.15, scrollTrigger : { trigger : ".cam-view-2",  start : "top bottom", end : "top top", scrub : true, immediateRender : false } }
    )
    .to
    (
      target, { x : -0.78, y : -0.03, z : -0.12, scrollTrigger : { trigger : ".cam-view-2",  start : "top bottom", end : "top top", scrub : true, immediateRender : false } }
    )
    .to
    (
      '.hero--scroller',
      {
        opacity : 0,
        y : '150%',
        scrollTrigger : { trigger : ".cam-view-2", start : "top bottom", end : "top center", scrub : 1, scrub : true, immediateRender : false }
      }
    )
    .to
    (
      '.hero--container',
      {
        opacity: 0, xPercent: '100', ease: "power4.out", scrollTrigger: { trigger: ".cam-view-2", start: "top bottom", end: "top top", scrub : true, immediateRender : false }
      }
    )
    .to
    (
      '.forever--text-bg',
      {
        opacity : 0.1, ease : "power4.inOut", scrollTrigger : { trigger : ".cam-view-2", start : "top bottom", end : 'top top', scrub : true, immediateRender : false }
      }
    )
    .fromTo
    (
      '.forever--container',
      { opacity : 0, x : '-110%' },
      { opacity : 1, x : '0%', ease : "power4.inOut", scrollTrigger : { trigger : ".cam-view-2", start : "top bottom", end : 'top top', scrub : true, immediateRender : false } }
    )
    .addLabel( "Forever" )
    .to( '.side-bar .unique', { opacity : 1, scale : 1.5, ease : "power4.inOut", duration : 2, scrollTrigger : { trigger : ".cam-view-1", start : "top bottom", end : 'top top', scrub : true, immediateRender : false } } )
    .to( '.side-bar .unique', { opacity : 0.5, scale : 1, ease : "power4.inOut", duration : 2, scrollTrigger : { trigger : ".cam-view-2", start : "top bottom", end : 'top top', scrub : true, immediateRender : false } } )
    .to( '.side-bar .forever', { opacity : 1, scale : 1.5, ease : "power4.inOut", duration : 2, scrollTrigger : { trigger : ".cam-view-2", start : "top bottom", end : 'top top', scrub : true, immediateRender : false } } )

    // // EMOTIONS SECTION
    .to
    (
      position,
      { x : -0.06, y : -1.15, z : 4.42, scrollTrigger : { trigger : ".cam-view-3", start : "top bottom", end : "top top", scrub : true, immediateRender : false } }
    )
    .to
    (
      target,
      { x : -0.01, y : 0.9, z : 0.07, scrollTrigger : { trigger : ".cam-view-3",  start : "top bottom", end : "top top", scrub : true, immediateRender : false } }
    )
    .to
    (
      '.forever--container',
      {
        opacity : 0, x : '-110%', ease : "power4.inOut",
        scrollTrigger : { trigger : ".cam-view-3", start : "top bottom", end : 'top top', scrub : true, immediateRender : false }
      }
    )
    .to
    (
      '.emotions--text-bg',
      {
        opacity : 0.1, ease : "power4.inOut",
        scrollTrigger : { trigger : ".cam-view-3", start : "top bottom", end : 'top top', scrub : true, immediateRender : false }
      }
    )
    .to
    (
      '.emotions--image',
      {
        opacity : 1.0, ease : "power4.inOut",
        scrollTrigger : { trigger : ".cam-view-3", start : "top bottom", end : 'top top', scrub : true, immediateRender : false }
      }
    )
    .fromTo
    (
      '.emotions--content',
      { opacity : 0, x : '200%', y : '130%' },
      {
        opacity : 1, x : '-75%', y : '0%', duration : 0.5, ease : "power4.inOut",
        scrollTrigger : { trigger : ".cam-view-3", start : "top bottom", end : "top top", scrub : true, immediateRender : false }
      }
    )
    .addLabel( "Emotions" )
    .to
    (
      '.side-bar .forever',
      {
        opacity : 0.5, scale : 1, ease : "power4.inOut", duration : 2,
        scrollTrigger : { trigger : ".cam-view-3", start : "top bottom", end : 'top top', scrub : true, immediateRender : false }
      }
    )
    .to
    (
      '.side-bar .emotions',
      {
        opacity : 1, scale : 1.5, ease : "power4.inOut", duration : 2,
        scrollTrigger : { trigger : ".cam-view-3", start : "top bottom", end : 'top top', scrub : true, immediateRender : false }
      }
    )
    .to
    (
      '.side-bar .forever',
      {
        opacity : 0.5, scale : 1, ease : "power4.inOut", duration : 2,
        scrollTrigger : { trigger : ".cam-view-1", start : "top bottom", end : 'top top', scrub : true, immediateRender : false }
      }
    )
  }

  document.querySelector('.button-scroll')?.addEventListener('click', () => {
      const element = document.querySelector('.cam-view-2')
      window.scrollTo({top: element?.getBoundingClientRect().top, left: 0, behavior: 'smooth'})
  })

  document.querySelector('.forever')?.addEventListener('click', () => {
      const element = document.querySelector('.cam-view-2')
      window.scrollTo({top: element?.getBoundingClientRect().top, left: 0, behavior: 'smooth'})
  })

  document.querySelector('.hero--scroller')?.addEventListener('click', () => {
      const element = document.querySelector('.cam-view-2')
      window.scrollTo({top: element?.getBoundingClientRect().top, left: 0, behavior: 'smooth'})
  })

  document.querySelector('.btn-customize')?.addEventListener('click', () => {
      exploreView.style.pointerEvents = "none"
      document.body.style.overflowY = "hidden"
      document.body.style.cursor = "grab"
      sidebar.style.display = "none"
      headerContainer.style.display = "none"
      footerContainer.style.display = "flex"
      configAnimation()
  })

  const tlExplore = gsap.timeline()

  function configAnimation()
  {
    let innerLightColor = "#FFFFFF";
    let outerLightColor = "#DDDDDD";

    tlExplore
    .to( position, { x : -0.17, y : -0.25, z : 8.5, duration : 2.5 } )
    .to( target, { x : 0, y : 0, z : 0, duration : 2.5 }, '-=2.5' )
    .to( '.emotions--content', { opacity : 0, x : '200%', duration : 1.5, ease : "power4.out", onComplete : onCompleteConfigAnimation }, '-=2.5' )
    .to( '.emotions--text-bg', { opacity : 0, x : '200%', duration : 1.5, ease : "power4.out" }, '-=2.5' )
    .to( '.emotions--image', { opacity : 0, y : '100%', duration : 1.5, ease : "power4.out" }, '-=2.5' )
    .to(
      document.body,
      {
        duration: 0.75,
        "--bg-color-inner": innerLightColor,
        "--bg-color-outer": outerLightColor
      },
      '-=2.5'
    )
    .fromTo( '.footer--menu', { opacity : 0, y : '150%' }, { opacity : 1, y : '0%', duration : 1.5 } )
  }

  function onCompleteConfigAnimation()
  {
    camView1.style.display = "none"
    camView2.style.display = "none"
    camView3.style.display = "none"

    previewContainer.style.display = "flex"
    exitContainer.style.display = "flex"
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

  document.querySelector( '.button--exit' )?.addEventListener
  (
    'click',
    () =>
    {
      camView1.style.display = "flex"
      camView2.style.display = "flex"
      camView3.style.display = "flex"
      headerContainer.style.display = "flex"
      emotionsImage.style.display = "flex"

      previewContainer.style.display = "none"
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

      exploreView.style.pointerEvents = "all"
      document.body.style.overflowY = "auto"
      document.body.style.cursor = "auto"
      sidebar.style.display = "block"
      exitConfigAnimation()
    }
  )

  const tlExit = gsap.timeline()

  // EXIT EVENT
  function exitConfigAnimation()
  {
    let innerLightColor = "#FFFFFF";

    gemMenu.classList.remove( 'show' )
    materialsMenu.classList.remove( 'show' )
    ringsMenu.classList.remove( 'show' )
    if ( document.querySelector( '.footer--menu li.active' ) )
    {
      document.querySelector( '.footer--menu li.active' )?.classList.remove( 'active' )
    }

    tlExit.to( position, { x : -0.06, y : -1.15, z : 4.42, duration : 1.2, ease : "power4.out" } )
    .to( target, { x : -0.01, y : 0.9, z : 0.07, duration : 1.2, ease : "power4.out" }, '-=1.2' )
    .to( '.footer--menu', { opacity : 0, y : '150%' }, '-=1.2' )
    .to( '.emotions--content', { opacity : 1, x : '0%', duration : 0.5, ease : "power4.out" }, '-=1.2' )
    .fromTo( '.emotions--image', { opacity : 0, y : '100%'}, { opacity : 1, y : '50%', duration : 0.5, ease : "power4.inOut" }, '-=1.2' )
    .to(
      document.body,
      {
        duration: 0.25,
        "--bg-color-inner": innerLightColor,
        "--bg-color-outer": innerLightColor
      },
      '-=1.2'
    )
  }
}

// -- CONFIGURATOR --

function setupConfigurator()
{
  const header = document.querySelector('.header')
  const camView1 =  document.querySelector('.cam-view-1')
  const camView2 =  document.querySelector('.cam-view-2')
  const camView3 =  document.querySelector('.cam-view-3')

  const exitContainer = document.querySelector('.exit--container')
  const gemMenu = document.querySelector('.gem--menu')
  const footerMenu = document.querySelector('.footer--menu')
  const materialsMenu = document.querySelector('.materials--menu')
  const ringsMenu = document.querySelector('.rings--menu')
  const configMaterial = document.querySelector('.config--material')
  const configGem = document.querySelector('.config--gem')
  const closeConfigMaterial = document.querySelector('.close-materials')
  const configRing = document.querySelector('.config--ring')
  const closeConfigGem = document.querySelector('.close-gems')
  const closeConfigRing = document.querySelector('.close-rings')
  const footerContainer = document.querySelector('.footer--container')

  let nightMode = false

  function toggleNightMode()
  {
    console.log("A")
    if(!nightMode){
        header.classList.add('night--mode--filter')
        camView1.classList.add('night--mode--filter')
        camView2.classList.add('night--mode--filter')
        camView3.classList.add('night--mode--filter')
        exitContainer.classList.add('night--mode--filter')
        footerMenu.classList.add('night--mode--filter')
        gsap.to(document.body, {
            duration: 0.75,
            "--bg-color-inner": innerDarkColor,
            "--bg-color-outer": outerDarkColor
        });
        nightMode = true
    } else{
        header.classList.remove('night--mode--filter')
        camView1.classList.remove('night--mode--filter')
        camView2.classList.remove('night--mode--filter')
        camView3.classList.remove('night--mode--filter')
        footerMenu.classList.remove('night--mode--filter')
        exitContainer.classList.remove('night--mode--filter')
        gsap.to(document.body, {
            duration: 0.75,
            "--bg-color-inner": innerLightColor,
            "--bg-color-outer": outerLightColor
        });
        nightMode = false
    }
  }

  // NIGHT MODE
  document.querySelector('.night--mode')?.addEventListener('click', () => {
      toggleNightMode()
  })

  let innerLightColor = "#FFFFFF";
  let outerLightColor = "#DDDDDD";
  let innerDarkColor = "#777777";
  let outerDarkColor = "#000000";

  // GEM MENU
  configGem.addEventListener('click', () => {
      gemMenu.classList.add('show')
      materialsMenu.classList.remove('show')
      ringsMenu.classList.remove('show')

      if (document.querySelector('.footer--menu li.active')){
          document.querySelector('.footer--menu li.active')?.classList.remove('active')
      }
      configGem.parentElement?.classList.add('active')
  })

  // DIAMOND COLORS
  document.querySelector('.ruby')?.addEventListener('click', () => {
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.ruby')?.classList.add('active')
  })
  document.querySelector('.white')?.addEventListener('click', () => {
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.white')?.classList.add('active')
  })
  document.querySelector('.emerald')?.addEventListener('click', () => {
      document.querySelector('.colors--list li.active')?.classList.remove('active')
      document.querySelector('.emerald')?.classList.add('active')
  })

  // MATERIALS MENU
  configMaterial.addEventListener('click', () => {
      materialsMenu.classList.add('show')
      gemMenu.classList.remove('show')
      ringsMenu.classList.remove('show')

      if (document.querySelector('.footer--menu li.active')){
          document.querySelector('.footer--menu li.active')?.classList.remove('active')
      }
      configMaterial.parentElement?.classList.add('active')
  })

  // MATERIALS COLOR
  document.querySelector('.silver')?.addEventListener('click', () => {
      document.querySelector('.materials--list li.active')?.classList.remove('active')
      document.querySelector('.silver')?.classList.add('active')
    })
  document.querySelector('.copper')?.addEventListener('click', () => {
      document.querySelector('.materials--list li.active')?.classList.remove('active')
      document.querySelector('.copper')?.classList.add('active')
    })

  document.querySelector('.gold')?.addEventListener('click', () => {
      document.querySelector('.materials--list li.active')?.classList.remove('active')
      document.querySelector('.gold')?.classList.add('active')
  })

  // CHANGE RING
  configRing.addEventListener('click', () => {
      ringsMenu.classList.add('show')
      materialsMenu.classList.remove('show')
      gemMenu.classList.remove('show')

      if (document.querySelector('.footer--menu li.active')){
          document.querySelector('.footer--menu li.active')?.classList.remove('active')
      }
      configRing.parentElement?.classList.add('active')
  })

  // CLOSE GEM MENU
  closeConfigGem.addEventListener('click', () => {
      gemMenu.classList.remove('show')

      if (document.querySelector('.footer--menu li.active')){
          document.querySelector('.footer--menu li.active')?.classList.remove('active')
      }
  })

  // CLOSE MATERIAL MENU
  closeConfigMaterial.addEventListener('click', () => {
      materialsMenu.classList.remove('show')

      if (document.querySelector('.footer--menu li.active')){
          document.querySelector('.footer--menu li.active')?.classList.remove('active')
      }
  })

  // CLOSE RING MENU
  closeConfigRing.addEventListener('click', () => {
      ringsMenu.classList.remove('show')

      if (document.querySelector('.footer--menu li.active')){
          document.querySelector('.footer--menu li.active')?.classList.remove('active')
      }
  })

  // Configuration state
  const state = {
    gem: "white",
    metal: "silver",
    ring: 1
  };

  const previewContainer = document.querySelector(".preview--image");
  const previewImage = previewContainer.querySelector("img");

  function updatePreview() {
    const { metal, gem, ring } = state;
    const imagePath = `/assets/jewelry/${metal}_${gem}_${ring}.png`;

    previewImage.src = imagePath;
    previewContainer.classList.add("show");

    clearTimeout(previewContainer._hideTimer);
    previewContainer._hideTimer = setTimeout(() => {
      previewContainer.classList.remove("show");
    }, 2500);
  }

  function setActive(target, groupSelector) {
    document.querySelectorAll(`${groupSelector} li.active`).forEach(li =>
      li.classList.remove("active")
    );
    target.classList.add("active");
  }

  function bindSelector(groupSelector, type, valueGetter) {
    document.querySelectorAll(`${groupSelector} li`).forEach(li => {
      li.addEventListener("click", () => {
        setActive(li, groupSelector);
        state[type] = valueGetter(li);
        updatePreview();
      });
    });
  }

  bindSelector(".colors--list", "gem", li => {
    if (li.classList.contains("ruby")) return "red";
    if (li.classList.contains("white")) return "white";
    if (li.classList.contains("emerald")) return "green";
    return state.gem;
  });

  bindSelector(".materials--list", "metal", li => {
    if (li.classList.contains("silver")) return "silver";
    if (li.classList.contains("gold")) return "gold";
    if (li.classList.contains("copper")) return "copper";
    return state.metal;
  });

  bindSelector(".rings--list", "ring", li => {
    if (li.classList.contains("ring1")) return 1;
    if (li.classList.contains("ring2")) return 2;
    if (li.classList.contains("ring3")) return 3;
    return state.ring;
  });

  previewContainer.style.display = "none"
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
  () =>
  {
    setupMainPage();
    setupConfigurator();
  }
);

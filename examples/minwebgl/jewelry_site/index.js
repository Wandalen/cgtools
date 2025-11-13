let firstLoad = true

// -- MAIN PAGE --

async function setupMainPage()
{
  const position = { x : 0.0, y : 0.0, z : 0.0 }
  const target = { x : 0.0, y : 0.0, z : 0.0 }

  const exploreView = document.querySelector('.cam-view-3')
  const canvasView = document.getElementById('webgi-canvas')
  const canvasContainer = document.getElementById('webgi-canvas-container')
  const exitContainer = document.querySelector('.exit--container')
  const loaderElement = document.querySelector('.loader')
  const sidebar = document.querySelector('.side-bar')
  const header = document.querySelector('.header')
  const camView1 =  document.querySelector('.cam-view-1')
  const camView2 =  document.querySelector('.cam-view-2')
  const camView3 =  document.querySelector('.cam-view-3')
  const footerContainer = document.querySelector('.footer--container')
  const gemMenu = document.querySelector('.gem--menu')
  const materialsMenu = document.querySelector('.materials--menu')
  const ringsMenu = document.querySelector('.rings--menu')

  // importer.addEventListener("onProgress", (ev) => {
  //     const progressRatio = (ev.loaded / ev.total)
  //     document.querySelector('.progress')?.setAttribute('style',`transform: scaleX(${progressRatio})`)
  // })

  window.addEventListener("load", (ev) => {
    if ( firstLoad )
    {
      introAnimation()
    }
    else
    {
      gsap.to('.loader', {x: '100%', duration: 0.8, ease: "power4.inOut", delay: 1})
    }
  })

  window.scrollTo(0,0)

  function introAnimation()
  {
    firstLoad = false
    const introTL = gsap.timeline()
    introTL
    .to( '.loader', { x: '100%', duration : 0.8, ease : "power4.inOut", delay : 1 } )
    .fromTo( position, { x : 3, y : -0.8, z : 1.2 }, { x : 1.28, y : -1.7, z : 5.86, duration : 4 }, '-=0.8' )
    .fromTo( target, { x : 2.5, y : -0.07, z : -0.1 }, { x : 0.91, y: 0.03, z : -0.25, duration : 4 }, '-=4' )
    .fromTo( '.header--container', { opacity : 0, y : '-100%' }, {opacity : 1, y : '0%', ease : "power1.inOut", duration : 0.8 }, '-=1' )
    .fromTo( '.hero--scroller', { opacity : 0, y : '150%' }, { opacity : 1, y : '0%', ease : "power4.inOut", duration : 1 }, '-=1' )
    .fromTo( '.hero--container', { opacity : 0, x : '100%' }, { opacity : 1, x : '0%', ease : "power4.inOut", duration : 1.8, onComplete : setupScrollAnimation }, '-=1' )
    .fromTo( '.side-bar', { opacity : 0, x : '50%' }, { opacity : 1, x : '0%', ease : "power4.inOut", duration : 2 }, '-=1' )
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
    // .to
    // (
    //   ring.rotation,
    //   {
    //     x : ( ringModel == 1 ) ? 0 : -Math.PI / 3, y : ( ringModel == 1 ) ? 0 : -0.92, z : ( ringModel == 1 ) ? Math.PI / 2 : 0,
    //     scrollTrigger : { trigger : ".cam-view-2",  start : "top bottom", end : "top top", scrub : true, immediateRender : false }
    //   }
    // )
    // .fromTo
    // (
    //   colorLerpValue,
    //   { x : 0 },
    //   {
    //     x : 1,
    //     scrollTrigger : { trigger : ".cam-view-2",  start : "top bottom", end : "top top", scrub : true, immediateRender : false },
    //     onUpdate :
    //     function()
    //     {
    //       // if( !usingCustomColors )
    //       // {
    //       //   silver.material.color.lerpColors( new Color( 0xfefefe ).convertSRGBToLinear(), new Color( 0xd28b8b ).convertSRGBToLinear(), colorLerpValue.x )
    //       //   gold.material.color.lerpColors( new Color( 0xe2bf7f ).convertSRGBToLinear(), new Color( 0xd28b8b ).convertSRGBToLinear(), colorLerpValue.x )
    //       //   for ( const o of diamondObjects )
    //       //   {
    //       //     o.material.color.lerpColors(new Color( 0xffffff ).convertSRGBToLinear(), new Color( 0x39cffe ).convertSRGBToLinear(), colorLerpValue.x )
    //       //   }
    //       // }
    //     }
    //   }
    // )
    .to
    (
      '.hero--scroller',
      {
        opacity : 0,
        y : '150%',
        scrollTrigger : { trigger : ".cam-view-2", start : "top bottom", end : "top center", scrub : 1, pin : '.hero--scroller--container' }
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
      '.forever--container', { opacity : 0, x : '-110%' },
      { opacity : 1, x : '0%', ease : "power4.inOut", scrollTrigger : { trigger : ".cam-view-2", start : "top bottom", end : 'top top', scrub : true, immediateRender : false } }
    )
    .addLabel( "Forever" )
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
    // .to
    // (
    //   ring.rotation,
    //   {
    //     x : ( ringModel == 1 ) ? 0 : 0.92 , y : ( ringModel == 1 ) ? 0 : 0.92, z : ( ringModel == 1 ) ? -Math.PI / 2 : Math.PI / 3,
    //     scrollTrigger : { trigger : ".cam-view-3", start : "top bottom", end : "top top", scrub : true, immediateRender : false }
    //   }
    // )
    // .fromTo
    // (
    //   colorLerpValue2,
    //   { x : 0 },
    //   {
    //     x : 1, scrollTrigger : { trigger : ".cam-view-3", start : "top bottom", end : "top top", scrub : true, immediateRender : false },
    //     onUpdate : function()
    //     {
    //       // if( !usingCustomColors )
    //       // {
    //       //   silver.material.color.lerpColors( new Color( 0xd28b8b ).convertSRGBToLinear(), new Color( 0xf7c478 ).convertSRGBToLinear(), colorLerpValue2.x )
    //       //   gold.material.color.lerpColors( new Color( 0xd28b8b ).convertSRGBToLinear(), new Color( 0xf7c478 ).convertSRGBToLinear(), colorLerpValue2.x )
    //       //   for ( const o of diamondObjects )
    //       //   {
    //       //     o.material.color.lerpColors( new Color( 0x39cffe ).convertSRGBToLinear(), new Color( 0xf70db1 ).convertSRGBToLinear(), colorLerpValue2.x )
    //       //   }
    //       // }
    //     }
    //   }
    // )
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
    .fromTo
    (
      '.emotions--content',
      { opacity : 0, y : '130%' },
      {
        opacity : 1, y : '0%', duration : 0.5, ease : "power4.inOut",
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
      '.side-bar .unique',
      {
        opacity : 1, scale : 1.5, ease : "power4.inOut", duration : 2,
        scrollTrigger : { trigger : ".cam-view-1", start : "top bottom", end : 'top top', scrub : true, immediateRender : false }
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
      footerContainer.style.display = "flex"
      configAnimation()
  })

  const tlExplore = gsap.timeline()

  function configAnimation()
  {
    tlExplore
    .to( position, { x : -0.17, y : -0.25, z : 8.5, duration : 2.5 } )
    .to( target, { x : 0, y : 0, z : 0, duration : 2.5 }, '-=2.5' )
    // .to( ring.rotation, { x : ( ringModel == 1 ) ? -Math.PI / 2 : 0, y : 0, z : ( ringModel == 1 ) ? -Math.PI / 2 : 0, duration : 2.5 }, '-=2.5' )
    .to( '.emotions--content', { opacity : 0, x : '130%', duration : 1.5, ease : "power4.out", onComplete : onCompleteConfigAnimation }, '-=2.5' )
    .fromTo( '.footer--menu', { opacity : 0, y : '150%' }, { opacity : 1, y : '0%', duration : 1.5 } )
  }

  let colorLerpValue = { x : 0 }
  let colorLerpValue2 = { x : 0 }

  function onCompleteConfigAnimation()
  {
    exitContainer.style.display = "flex"
    // if(camera.controls)
    //   {
    //     camera.controls.enabled = true
    //     camera.controls.autoRotate = true
    //     camera.controls.minDistance = 5
    //     camera.controls.maxDistance = 13
    //     camera.controls.enablePan = false
    //     camera.controls.screenSpacePanning = false
    // }
    // dof.pass!.passObject.enabled = false
  }

  document.querySelector( '.button--exit' )?.addEventListener
  (
    'click',
    () =>
    {
      exploreView.style.pointerEvents = "all"
      // canvasView.style.pointerEvents = "none"
      // canvasContainer.style.zIndex = "unset"
      document.body.style.overflowY = "auto"
      exitContainer.style.display = "none"
      document.body.style.cursor = "auto"
      sidebar.style.display = "block"
      footerContainer.style.display = "none"
      exitConfigAnimation()

      // customScrollingEnabled = true;
    }
  )

  const tlExit = gsap.timeline()

  // EXIT EVENT
  function exitConfigAnimation()
  {
    // if ( camera.controls )
    // {
    //   camera.controls.enabled = true
    //   camera.controls.autoRotate = false
    //   camera.controls.minDistance = 0
    //   camera.controls.maxDistance = Infinity
    // }

    // dof.pass!.passObject.enabled = true

    gemMenu.classList.remove( 'show' )
    materialsMenu.classList.remove( 'show' )
    ringsMenu.classList.remove( 'show' )
    if ( document.querySelector( '.footer--menu li.active' ) )
    {
      document.querySelector( '.footer--menu li.active' )?.classList.remove( 'active' )
    }

    tlExit.to( position, { x : -0.06, y : -1.15, z : 4.42, duration : 1.2, ease : "power4.out" } )
    .to( target, { x : -0.01, y : 0.9, z : 0.07, duration : 1.2, ease : "power4.out" }, '-=1.2' )
    // .to( ring.rotation, { x : ( ringModel == 1 ) ? 0 : 0.92 , y : ( ringModel == 1 ) ? 0 : 0.92, z : ( ringModel == 1 ) ? -Math.PI / 2 : Math.PI / 3 }, '-=1.2' )
    .to( '.footer--menu', { opacity : 0, y : '150%' }, '-=1.2' )
    .to( '.emotions--content', { opacity : 1, x : '0%', duration : 0.5, ease : "power4.out" }, '-=1.2' )
  }
}

// -- CONFIGURATOR --

function setup_configurator()
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

  let nightMode = false

  function toggleNightMode()
  {
    if(!nightMode){
        header.classList.add('night--mode--filter')
        camView1.classList.add('night--mode--filter')
        camView2.classList.add('night--mode--filter')
        camView3.classList.add('night--mode--filter')
        exitContainer.classList.add('night--mode--filter')
        footerMenu.classList.add('night--mode--filter')
        gsap.to(document.body, {
            duration: 0.75,
            "--bg-color-inner": inner_dark_color,
            "--bg-color-outer": outer_dark_color
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
            "--bg-color-inner": inner_light_color,
            "--bg-color-outer": outer_light_color
        });
        nightMode = false
    }
  }

  // NIGHT MODE
  document.querySelector('.night--mode')?.addEventListener('click', () => {
      toggleNightMode()
  })

  let inner_light_color = "#FFFFFF";
  let outer_light_color = "#DDDDDD";
  let inner_dark_color = "#777777";
  let outer_dark_color = "#000000";

  gsap.to(document.body, {
      duration: 0.75,
      "--bg-color-inner": inner_light_color,
      "--bg-color-outer": outer_light_color
  });

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

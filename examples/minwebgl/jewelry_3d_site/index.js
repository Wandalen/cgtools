export let uiState =
{
  gem : "white",
  metal : "silver",
  ring : 0,
  changed :
  [
    "gem",
    "metal",
    "ring"
  ]
};

export function getUiState()
{
  return uiState;
}

export function isChanged()
{
  return uiState.changed.length > 0;
}

export function clearChanged()
{
  uiState.changed.length = 0;
}

// function introAnimation()
// {
//   firstLooad = false
//   const introTL = gsap.timeline()
//   introTL
//   .to( '.loader', { x: '100%', duration : 0.8, ease : "power4.inOut", delay : 1 } )
//   .fromTo( position, { x : isMobile ? 3 : 3, y : isMobile ? -0.8 : -0.8, z : isMobile ? 1.2 : 1.2 }, { x : isMobile ? 1.28 : 1.28, y : isMobile ? -1.7 : -1.7, z : isMobile ? 5.86 : 5.86, duration : 4, onUpdate }, '-=0.8' )
//   .fromTo( target, { x : isMobile ? 2.5 : 2.5, y : isMobile ? -0.07 : -0.07, z : isMobile ? -0.1 : -0.1 }, { x : isMobile ? -0.21 : 0.91, y: isMobile ? 0.03 : 0.03, z : isMobile ? -0.25 : -0.25, duration : 4, onUpdate }, '-=4' )
//   .fromTo( '.header--container', { opacity : 0, y : '-100%' }, {opacity : 1, y : '0%', ease : "power1.inOut", duration : 0.8 }, '-=1' )
//   .fromTo( '.hero--scroller', { opacity : 0, y : '150%' }, { opacity : 1, y : '0%', ease : "power4.inOut", duration : 1 }, '-=1' )
//   .fromTo( '.hero--container', { opacity : 0, x : '100%' }, { opacity : 1, x : '0%', ease : "power4.inOut", duration : 1.8, onComplete : setupScrollAnimation }, '-=1' )
//   .fromTo( '.side-bar', { opacity : 0, x : '50%' }, { opacity : 1, x : '0%', ease : "power4.inOut", duration : 2 }, '-=1' )
//   .to( '.side-bar .unique', { opacity : 1, scale : 1.5, ease : "power4.inOut", duration : 2 }, '-=1' )
// }

// function setupScrollAnimation()
// {
//   document.body.style.overflowY = "scroll"

//   const tl = gsap.timeline( { default : { ease: 'none' } } )

//   // FOREVER
//   tl
//   .to
//   (
//     position, { x : -1.83, y : -0.14, z : 6.15, scrollTrigger : { trigger : ".cam-view-2",  start : "top bottom", end : "top top", scrub : true, immediateRender : false }, onUpdate }
//   )
//   .to
//   (
//     target, { x : isMobile ? 0 : -0.78, y : isMobile ? 1.5 : -0.03, z : -0.12, scrollTrigger : { trigger : ".cam-view-2",  start : "top bottom", end : "top top", scrub : true, immediateRender : false } }
//   )
//   .to
//   (
//     ring.rotation,
//     {
//       x : ( ringModel == 1 ) ? 0 : -Math.PI / 3, y : ( ringModel == 1 ) ? 0 : -0.92 , z : ( ringModel == 1 ) ? Math.PI / 2 : 0,
//       scrollTrigger : { trigger : ".cam-view-2",  start : "top bottom", end : "top top", scrub : true, immediateRender : false }
//     }
//   )
//   .fromTo
//   (
//     colorLerpValue,
//     { x : 0 },
//     {
//       x : 1,
//       scrollTrigger : { trigger : ".cam-view-2",  start : "top bottom", end : "top top", scrub : true, immediateRender : false },
//       onUpdate :
//       function()
//       {
//         if( !usingCustomColors )
//         {
//           silver.material.color.lerpColors( new Color( 0xfefefe ).convertSRGBToLinear(), new Color( 0xd28b8b ).convertSRGBToLinear(), colorLerpValue.x )
//           gold.material.color.lerpColors( new Color( 0xe2bf7f ).convertSRGBToLinear(), new Color( 0xd28b8b ).convertSRGBToLinear(), colorLerpValue.x )
//           for ( const o of diamondObjects )
//           {
//             o.material.color.lerpColors(new Color( 0xffffff ).convertSRGBToLinear(), new Color( 0x39cffe ).convertSRGBToLinear(), colorLerpValue.x )
//           }
//         }
//       }
//     }
//   )
//   .to
//   (
//     '.hero--scroller',
//     {
//       opacity : 0,
//       y : '150%',
//       scrollTrigger : { trigger : ".cam-view-2", start : "top bottom", end : "top center", scrub : 1, immediateRender : false, pin : '.hero--scroller--container' }
//     }
//   )
//   .to
//   (
//     '.hero--container',
//     {
//       opacity: 0, xPercent: '100', ease: "power4.out", scrollTrigger: { trigger: ".cam-view-2", start: "top bottom", end: "top top", scrub: 1, immediateRender: false, }
//     }
//   )
//   .to
//   (
//     '.forever--text-bg',
//     {
//       opacity : 0.1, ease : "power4.inOut", scrollTrigger : { trigger : ".cam-view-2", start : "top bottom", end : 'top top', scrub : 1, immediateRender : false, }
//     }
//   )

//   .fromTo('.forever--container', {opacity: 0, x: '-110%'}, {opacity: 1, x: '0%', ease: "power4.inOut",
//       scrollTrigger: { trigger: ".cam-view-2", start: "top bottom", end: 'top top', scrub: 1, immediateRender: false,
//   }})
//   .addLabel("Forever")
//   .to('.side-bar .unique', { opacity: 0.5, scale: 1, ease: "power4.inOut", duration: 2, scrollTrigger: { trigger: ".cam-view-2", start: "top bottom", end: 'top top', scrub: 1, immediateRender: false,}})
//   .to('.side-bar .forever', { opacity: 1, scale: 1.5, ease: "power4.inOut", duration: 2, scrollTrigger: { trigger: ".cam-view-2", start: "top bottom", end: 'top top', scrub: 1, immediateRender: false}})


//   // // EMOTIONS SECTION
//   .to(position,  {x: -0.06, y: -1.15, z: 4.42,
//       scrollTrigger: { trigger: ".cam-view-3",  start: "top bottom", end: "top top", scrub: true, immediateRender: false,
//   }, onUpdate
//   })
//   .to(target, {x: -0.01, y: 0.9, z: 0.07,
//       scrollTrigger: { trigger: ".cam-view-3",  start: "top bottom", end: "top top", scrub: true, immediateRender: false }, onUpdate
//   })
//   .to(ring.rotation,{x: (ringModel == 1) ? 0 :0.92 , y:(ringModel == 1) ? 0 : 0.92, z: (ringModel == 1) ? -Math.PI /2 : Math.PI/3,
//       scrollTrigger: { trigger: ".cam-view-3",  start: "top bottom", end: "top top", scrub: true, immediateRender: false }
//   })
//   .fromTo(colorLerpValue2, {x:0}, {x:1,
//       scrollTrigger: { trigger: ".cam-view-3",  start: "top bottom", end: "top top", scrub: true, immediateRender: false }
//       , onUpdate: function() {


//           if(!usingCustomColors){
//               silver.material.color.lerpColors(new Color(0xd28b8b).convertSRGBToLinear(), new Color(0xf7c478).convertSRGBToLinear(), colorLerpValue2.x)
//               gold.material.color.lerpColors(new Color(0xd28b8b).convertSRGBToLinear(), new Color(0xf7c478).convertSRGBToLinear(), colorLerpValue2.x)
//               for (const o of diamondObjects) {
//                   o.material.color.lerpColors(new Color(0x39cffe).convertSRGBToLinear(), new Color(0xf70db1).convertSRGBToLinear(), colorLerpValue2.x)
//               }
//           }
//   }})
//   .to('.forever--container', {opacity: 0, x: '-110%', ease: "power4.inOut",
//       scrollTrigger: { trigger: ".cam-view-3", start: "top bottom", end: 'top top', scrub: 1, immediateRender: false
//   }})
//   .to('.emotions--text-bg', {opacity: 0.1, ease: "power4.inOut",
//       scrollTrigger: { trigger: ".cam-view-3", start: "top bottom", end: 'top top', scrub: 1, immediateRender: false,
//   }})
//   .fromTo('.emotions--content', {opacity: 0, y: '130%'}, {opacity: 1, y: '0%', duration: 0.5, ease: "power4.inOut",
//       scrollTrigger: { trigger: ".cam-view-3", start: "top bottom", end: "top top", scrub: 1, immediateRender: false
//   }})

//   .addLabel("Emotions")
//   .to('.side-bar .forever', { opacity: 0.5, scale: 1, ease: "power4.inOut", duration: 2, scrollTrigger: { trigger: ".cam-view-3", start: "top bottom", end: 'top top', scrub: 1, immediateRender: false,}})
//   .to('.side-bar .emotions', { opacity: 1, scale: 1.5, ease: "power4.inOut", duration: 2, scrollTrigger: { trigger: ".cam-view-3", start: "top bottom", end: 'top top', scrub: 1, immediateRender: false}})

// }


document.addEventListener
(
  "DOMContentLoaded", () =>
  {
    const headerContainer = document.querySelector( '.header--container' )
    const gemMenu = document.querySelector( '.gem--menu' )
    const footerContainer = document.querySelector( '.footer--container' )
    const footerMenu = document.querySelector( '.footer--menu' )
    const materialsMenu = document.querySelector( '.materials--menu' )
    const ringsMenu = document.querySelector( '.rings--menu' )
    const configMaterial = document.querySelector( '.config--material' )
    const configGem = document.querySelector( '.config--gem' )
    const closeConfigMaterial = document.querySelector( '.close-materials' )
    const configRing = document.querySelector( '.config--ring' )
    const closeConfigGem = document.querySelector( '.close-gems' )
    const closeConfigRing = document.querySelector( '.close-rings' )

    // GEM MENU
    configGem.addEventListener
    (
      'click',
      () =>
      {
        gemMenu.classList.add( 'show' )
        materialsMenu.classList.remove( 'show' )
        ringsMenu.classList.remove( 'show' )

        if ( document.querySelector( '.footer--menu li.active' ) )
        {
          document.querySelector( '.footer--menu li.active' )?.classList.remove( 'active' )
        }
        configGem.parentElement?.classList.add( 'active' )
      }
    )

    // DIAMOND COLORS
    document.querySelector( '.red' )?.addEventListener
    (
      'click',
      () =>
      {
        document.querySelector( '.colors--list li.active')?.classList.remove('active')
        document.querySelector( '.red')?.classList.add('active')
      }
    )
    document.querySelector( '.green' )?.addEventListener
    (
      'click',
      () =>
      {
        document.querySelector( '.colors--list li.active')?.classList.remove('active')
        document.querySelector( '.green')?.classList.add('active')
      }
    )
    document.querySelector( '.light_blue' )?.addEventListener
    (
      'click',
      () =>
      {
        document.querySelector( '.colors--list li.active')?.classList.remove('active')
        document.querySelector( '.light_blue')?.classList.add('active')
      }
    )
    document.querySelector( '.turquoise' )?.addEventListener
    (
      'click',
      () =>
      {
        document.querySelector( '.colors--list li.active')?.classList.remove('active')
        document.querySelector( '.turquoise')?.classList.add('active')
      }
    )
    document.querySelector( '.yellow' )?.addEventListener
    (
      'click',
      () =>
      {
        document.querySelector( '.colors--list li.active')?.classList.remove('active')
        document.querySelector( '.yellow')?.classList.add('active')
      }
    )
    document.querySelector( '.blue' )?.addEventListener
    (
      'click',
      () =>
      {
        document.querySelector( '.colors--list li.active')?.classList.remove('active')
        document.querySelector( '.blue')?.classList.add('active')
      }
    )
    document.querySelector( '.orange' )?.addEventListener
    (
      'click',
      () =>
      {
        document.querySelector( '.colors--list li.active')?.classList.remove('active')
        document.querySelector( '.orange')?.classList.add('active')
      }
    )
    document.querySelector( '.violet' )?.addEventListener
    (
      'click',
      () =>
      {
        document.querySelector( '.colors--list li.active')?.classList.remove('active')
        document.querySelector( '.violet')?.classList.add('active')
      }
    )
    document.querySelector( '.pink' )?.addEventListener
    (
      'click',
      () =>
      {
        document.querySelector( '.colors--list li.active')?.classList.remove('active')
        document.querySelector( '.pink')?.classList.add('active')
      }
    )
    document.querySelector( '.black' )?.addEventListener
    (
      'click',
      () =>
      {
        document.querySelector( '.colors--list li.active')?.classList.remove('active')
        document.querySelector( '.black')?.classList.add('active')
      }
    )
    document.querySelector( '.white' )?.addEventListener
    (
      'click',
      () =>
      {
        document.querySelector( '.colors--list li.active')?.classList.remove('active')
        document.querySelector( '.white')?.classList.add('active')
      }
    )

    // MATERIALS MENU
    configMaterial.addEventListener
    (
      'click',
      () =>
      {
        materialsMenu.classList.add( 'show' )
        gemMenu.classList.remove( 'show' )
        ringsMenu.classList.remove( 'show' )

        if ( document.querySelector( '.footer--menu li.active' ) )
        {
          document.querySelector( '.footer--menu li.active' )?.classList.remove( 'active' )
        }
        configMaterial.parentElement?.classList.add( 'active' )
      }
    )

    // MATERIALS COLOR
    document.querySelector( '.silver' )?.addEventListener
    (
      'click',
      () =>
      {
        document.querySelector( '.materials--list li.active' )?.classList.remove( 'active' )
        document.querySelector( '.silver' )?.classList.add( 'active' )
      }
    )
    document.querySelector( '.copper' )?.addEventListener
    (
      'click',
      () =>
      {
        document.querySelector( '.materials--list li.active' )?.classList.remove( 'active' )
        document.querySelector( '.copper' )?.classList.add( 'active' )
      }
    )
    document.querySelector( '.gold' )?.addEventListener
    (
      'click',
      () =>
      {
        document.querySelector( '.materials--list li.active' )?.classList.remove( 'active' )
        document.querySelector( '.gold' )?.classList.add( 'active' )
      }
    )

    // CHANGE RING
    configRing.addEventListener
    (
      'click',
      () =>
      {
        ringsMenu.classList.add( 'show' )
        materialsMenu.classList.remove( 'show' )
        gemMenu.classList.remove( 'show' )

        if ( document.querySelector( '.footer--menu li.active' ) )
        {
          document.querySelector( '.footer--menu li.active' )?.classList.remove( 'active' )
        }
        configRing.parentElement?.classList.add( 'active' )
      }
    )

    // CLOSE GEM MENU
    closeConfigGem.addEventListener
    (
      'click',
      () =>
      {
        gemMenu.classList.remove('show')

        if ( document.querySelector( '.footer--menu li.active' ) )
        {
          document.querySelector( '.footer--menu li.active' )?.classList.remove( 'active' )
        }
      }
    )

    // CLOSE MATERIAL MENU
    closeConfigMaterial.addEventListener
    (
      'click',
      () =>
      {
        materialsMenu.classList.remove( 'show' )

        if ( document.querySelector( '.footer--menu li.active' ) )
        {
          document.querySelector( '.footer--menu li.active' )?.classList.remove( 'active' )
        }
      }
    )

    // CLOSE RING MENU
    closeConfigRing.addEventListener
    (
      'click',
      () =>
      {
        ringsMenu.classList.remove( 'show' )

        if ( document.querySelector( '.footer--menu li.active' ) )
        {
          document.querySelector( '.footer--menu li.active' )?.classList.remove( 'active' )
        }
      }
    )

    function setActive( target, groupSelector )
    {
      document.querySelectorAll( `${groupSelector} li.active` )
      .forEach
      (
        li => li.classList.remove( "active" )
      );
      target.classList.add( "active" );
    }

    function bindSelector( groupSelector, type, valueGetter )
    {
      document.querySelectorAll( `${groupSelector} li` )
      .forEach
      (
        li =>
        {
          li.addEventListener
          (
            "click",
            () =>
            {
              setActive( li, groupSelector );
              uiState[ type ] = valueGetter( li );
              uiState.changed.push( type );
            }
          );
        }
      );
    }

    bindSelector
    (
      ".colors--list",
      "gem",
      li =>
      {
        if ( li.classList.contains( "white" ) ) return "white";
        if ( li.classList.contains( "black" ) ) return "black";
        if ( li.classList.contains( "red" ) ) return "red";
        if ( li.classList.contains( "orange" ) ) return "orange";
        if ( li.classList.contains( "yellow" ) ) return "yellow";
        if ( li.classList.contains( "green" ) ) return "green";
        if ( li.classList.contains( "turquoise" ) ) return "turquoise";
        if ( li.classList.contains( "light_blue" ) ) return "light_blue";
        if ( li.classList.contains( "blue" ) ) return "blue";
        if ( li.classList.contains( "violet" ) ) return "violet";
        if ( li.classList.contains( "pink" ) ) return "pink";
        return uiState.gem;
      }
    );

    bindSelector
    (
      ".materials--list",
      "metal",
      li =>
      {
        if ( li.classList.contains( "silver" ) ) return "silver";
        if ( li.classList.contains( "gold" ) ) return "gold";
        if ( li.classList.contains( "copper" ) ) return "copper";
        return uiState.metal;
      }
    );

    bindSelector
    (
      ".rings--list",
      "ring",
      li =>
      {
        if ( li.classList.contains( "ring0" ) ) return 0;
        if ( li.classList.contains( "ring1" ) ) return 1;
        if ( li.classList.contains( "ring2" ) ) return 2;
        return uiState.ring;
      }
    );
  }
);

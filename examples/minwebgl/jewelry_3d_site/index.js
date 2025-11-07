export let uiState =
{
  lightMode : "light",
  gem : "white",
  metal : "silver",
  ring : 0,
  changed :
  [
    "lightMode",
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

    let nightMode = false

    // NIGHT MODE
    document.querySelector( '.night--mode' )?.addEventListener( 'click', () => { toggleNightMode() } )

    let inner_light_color = "#FFFFFF";
    let outer_light_color = "#DDDDDD";
    let inner_dark_color = "#777777";
    let outer_dark_color = "#000000";

    gsap.to
    (
      document.body,
      {
        duration : 0.75,
        "--bg-color-inner" : inner_light_color,
        "--bg-color-outer" : outer_light_color
      }
    );

    function toggleNightMode()
    {
      if( !nightMode )
      {
        footerMenu.classList.add( 'night--mode--filter' )
        headerContainer.classList.add( 'night--mode--filter' )
        gsap.to
        (
          document.body,
          {
            duration : 0.75,
            "--bg-color-inner" : inner_dark_color,
            "--bg-color-outer" : outer_dark_color
          }
        );
        nightMode = true
        uiState["lightMode"] = "dark";
      }
      else
      {
        footerMenu.classList.remove( 'night--mode--filter' )
        headerContainer.classList.remove( 'night--mode--filter' )
        gsap.to
        (
          document.body,
          {
            duration : 0.75,
            "--bg-color-inner" : inner_light_color,
            "--bg-color-outer" : outer_light_color
          }
        );
        nightMode = false
        uiState[ "lightMode" ] = "light";
      }
      uiState.changed.push( "lightMode" );
    }

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

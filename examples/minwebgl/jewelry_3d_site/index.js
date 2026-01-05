export let uiState =
{
  gem : "white",
  metal : "silver",
  ring : 0,
  gemCustomColor : [ 1.0, 1.0, 1.0 ],
  gemMultiplier : 1.0,
  metalCustomColor : [ 0.753, 0.753, 0.753 ],
  metalMultiplier : 1.2,
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

export function enableDebugControls()
{
  const colorControls = document.querySelector( '.color-controls--container' );
  if ( colorControls )
  {
    colorControls.style.display = 'flex';

    // Setup event listeners now that controls are visible
    setupColorPickerListeners();
  }
}

function setupColorPickerListeners()
{
  // Helper function to convert hex color to RGB array
  function hexToRgb( hex )
  {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec( hex );
    return result ?
    [
      parseInt( result[1], 16 ) / 255,
      parseInt( result[2], 16 ) / 255,
      parseInt( result[3], 16 ) / 255
    ] : [ 1.0, 1.0, 1.0 ];
  }

  // Gem color picker
  const gemColorPicker = document.getElementById( 'gem-color-picker' );
  const gemMultiplier = document.getElementById( 'gem-multiplier' );
  const gemMultiplierValue = document.getElementById( 'gem-multiplier-value' );

  if ( gemColorPicker && gemMultiplier && gemMultiplierValue )
  {
    gemColorPicker.addEventListener
    (
      'input',
      ( e ) =>
      {
        const rgb = hexToRgb( e.target.value );
        uiState.gemCustomColor = rgb;
        uiState.gem = "custom";
        uiState.changed.push( "gem" );
      }
    );

    gemMultiplier.addEventListener
    (
      'input',
      ( e ) =>
      {
        uiState.gemMultiplier = parseFloat( e.target.value );
        gemMultiplierValue.textContent = uiState.gemMultiplier.toFixed( 1 );
        uiState.changed.push( "gem" );
      }
    );
  }

  // Metal color picker
  const metalColorPicker = document.getElementById( 'metal-color-picker' );
  const metalMultiplier = document.getElementById( 'metal-multiplier' );
  const metalMultiplierValue = document.getElementById( 'metal-multiplier-value' );

  if ( metalColorPicker && metalMultiplier && metalMultiplierValue )
  {
    metalColorPicker.addEventListener
    (
      'input',
      ( e ) =>
      {
        const rgb = hexToRgb( e.target.value );
        uiState.metalCustomColor = rgb;
        uiState.metal = "custom";
        uiState.changed.push( "metal" );
      }
    );

    metalMultiplier.addEventListener
    (
      'input',
      ( e ) =>
      {
        uiState.metalMultiplier = parseFloat( e.target.value );
        metalMultiplierValue.textContent = uiState.metalMultiplier.toFixed( 1 );
        uiState.changed.push( "metal" );
      }
    );
  }
}

async function replaceSVG( svgPath, selector )
{
  let svg = document.querySelector( selector );
  if ( !svg ) return;

  const response = await fetch( svgPath );
  const svgText = await response.text();

  const parser = new DOMParser();
  const newSvgDoc = parser.parseFromString( svgText, 'image/svg+xml' );
  const newSvg = newSvgDoc.documentElement;

  if ( svg.hasAttribute( 'width' ) )
  {
    newSvg.setAttribute('width', svg.getAttribute( 'width' ) );
  }
  if ( svg.hasAttribute( 'height' ) )
  {
    newSvg.setAttribute( 'height', svg.getAttribute( 'height' ) );
  }

  svg.classList.forEach( cls => newSvg.classList.add( cls ) );

  try
  {
    svg.outerHTML = newSvg.outerHTML;
  }
  catch
  {

  }
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

    replaceSVG( "./static/images/jewelry_site/icons/gem.svg", ".image--gem" )
    replaceSVG( "./static/images/jewelry_site/icons/metal.svg", ".image--material" )
    replaceSVG( "./static/images/jewelry_site/icons/ring.svg", ".image--ring" )

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
        if ( li.classList.contains( "red" ) ) return "red";
        if ( li.classList.contains( "orange" ) ) return "orange";
        if ( li.classList.contains( "yellow" ) ) return "yellow";
        if ( li.classList.contains( "green" ) ) return "green";
        if ( li.classList.contains( "turquoise" ) ) return "turquoise";
        if ( li.classList.contains( "blue" ) ) return "blue";
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
        if ( li.classList.contains( "ring3" ) ) return 3;
        if ( li.classList.contains( "ring4" ) ) return 4;
        return uiState.ring;
      }
    );
  }
);

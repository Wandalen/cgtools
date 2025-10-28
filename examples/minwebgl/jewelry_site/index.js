document.addEventListener("DOMContentLoaded", () => {
    const headerContainer = document.querySelector('.header--container')
    const gemMenu = document.querySelector('.gem--menu')
    const footerContainer = document.querySelector('.footer--container')
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

    function toggleNightMode(){
        if(!nightMode){
            footerMenu.classList.add('night--mode--filter')
            headerContainer.classList.add('night--mode--filter')
            gsap.to(document.body, {
                duration: 0.75,
                "--bg-color-inner": inner_dark_color,
                "--bg-color-outer": outer_dark_color
            });
            nightMode = true
        } else{
            footerMenu.classList.remove('night--mode--filter')
            headerContainer.classList.remove('night--mode--filter')
            gsap.to(document.body, {
                duration: 0.75,
                "--bg-color-inner": inner_light_color,
                "--bg-color-outer": outer_light_color
            });
            nightMode = false
        }
    }

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
});

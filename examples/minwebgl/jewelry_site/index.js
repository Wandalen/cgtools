document.addEventListener("DOMContentLoaded", () => {
    const headerContainer = document.querySelector('.header--container')
    const gemMenu = document.querySelector('.gem--menu')
    const footerContainer = document.querySelector('.footer--container')
    const footerMenu = document.querySelector('.footer--menu')
    const materialsMenu = document.querySelector('.materials--menu')
    const configMaterial = document.querySelector('.config--material')
    const configGem = document.querySelector('.config--gem')
    const closeConfigMaterial = document.querySelector('.close-materials')
    const configRing = document.querySelector('.config--ring')
    const closeConfigGem = document.querySelector('.close-gems')
    let nightMode = false
    let ringModel = 1

    document.querySelector('.btn-customize')?.addEventListener('click', () => {
        exploreView.style.pointerEvents = "none"
        canvasView.style.pointerEvents = "all"
        canvasContainer.style.zIndex = "1"
        document.body.style.overflowY = "hidden"
        document.body.style.cursor = "grab"
        sidebar.style.display = "none"
        footerContainer.style.display = "flex"
        configAnimation()
    })

    const tlExplore = gsap.timeline()

    function configAnimation(){

        tlExplore.to(position,{x: -0.17, y: -0.25, z: 8.5, duration: 2.5, onUpdate})
        .to(target, {x: 0, y: 0, z: 0, duration: 2.5, onUpdate}, '-=2.5')

        .to(ring.rotation,{x: (ringModel == 1) ? -Math.PI/2: 0, y: 0, z: (ringModel == 1) ? -Math.PI/2 : 0, duration: 2.5}, '-=2.5')
        .to('.emotions--content', {opacity: 0, x: '130%', duration: 1.5, ease: "power4.out", onComplete: onCompleteConfigAnimation}, '-=2.5')
        .fromTo('.footer--menu',{opacity: 0, y:'150%'}, {opacity: 1, y: '0%', duration: 1.5})

    }

    function onCompleteConfigAnimation(){
        headerContainer.style.display = "flex"
    }

    // NIGHT MODE
    document.querySelector('.night--mode')?.addEventListener('click', () => {
        toggleNightMode()
    })

    gsap.to(document.body, {
        duration: 0.75,
        "--bg-color-inner": "#FFDEDE",
        "--bg-color-outer": "#E9B2B0"
    });

    function toggleNightMode(){
        if(!nightMode){
            footerMenu.classList.add('night--mode--filter')
            headerContainer.classList.add('night--mode--filter')
            gsap.to(document.body, {
                duration: 0.75,
                "--bg-color-inner": "#720ea0",
                "--bg-color-outer": "#37084c"
            });
            nightMode = true
        } else{
            footerMenu.classList.remove('night--mode--filter')
            headerContainer.classList.remove('night--mode--filter')
            gsap.to(document.body, {
                duration: 0.75,
                "--bg-color-inner": "#FFDEDE",
                "--bg-color-outer": "#E9B2B0"
            });
            nightMode = false
        }
    }

    // GEM MENU
    configGem.addEventListener('click', () => {
        gemMenu.classList.add('show')
        materialsMenu.classList.remove('show')

        const gemCameraAnimation = gsap.timeline()

        gemCameraAnimation.to(position, {x: 1.6, y: 3.66, z: 2.55, duration: 1.5, onUpdate})
        .to(target,{x: isMobile ? 0 : -0.01, y: isMobile ? 0.5 : 0.89, z: -0.09, duration: 1.5}, '-=1.5')
        
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
        gsap.timeline().to(position,{x: -0.17, y: -0.25, z: 8.5, duration: 2.5, onUpdate})
        .to(target, {x: 0, y: 0, z: 0, duration: 2.5, onUpdate}, '-=2.5')
        
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

    // CLOSE GEM MENU
    closeConfigGem.addEventListener('click', () => {
        gemMenu.classList.remove('show')

        gsap.timeline().to(position,{x: -0.17, y: -0.25, z: 8.5, duration: 2.5, onUpdate})
        .to(target, {x: 0, y: 0, z: 0, duration: 2.5, onUpdate}, '-=2.5')
       
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

    // CHANGE RING
    configRing.addEventListener('click', () => {
        
           
        if (document.querySelector('.footer--menu li.active')){
            document.querySelector('.footer--menu li.active')?.classList.remove('active')
        }
    })
});
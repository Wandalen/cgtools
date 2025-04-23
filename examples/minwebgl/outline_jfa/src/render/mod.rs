mod shaders;
mod camera;

pub struct Renderer{    
    
}

impl Reader{
    pub fn new() -> Self{
        Self{

        }
    }
}

struct Viewport{
    width: u32,
    height: u32
}

impl Viewport{
    fn new(width: u32, height: u32) -> Self{
        Self{
            width,
            height
        }
    }
}
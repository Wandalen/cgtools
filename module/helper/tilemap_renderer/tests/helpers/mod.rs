use tilemap_renderer::assets::Assets;

pub fn empty_assets() -> Assets
{
  Assets
  {
    fonts : vec![],
    images : vec![],
    sprites : vec![],
    geometries : vec![],
    gradients : vec![],
    patterns : vec![],
    clip_masks : vec![],
    paths : vec![],
  }
}

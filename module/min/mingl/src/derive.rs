/// Internal namespace.
mod private
{
  // use crate::*;

}

crate::mod_interface!
{
  reuse ::derive_tools;
  reuse ::former;
  // The crate name itself must stay exposed alongside the reused items: the `Former`
  // derive expands to `former::`-prefixed paths (not `::former::`), so downstream
  // derive sites that get their scope from mingl's exposed namespace (e.g. minwebgl's
  // `ShaderSource`) need `former` as a nameable module, which `reuse` alone does not
  // provide. Removable only if upstream `former` switches to absolute paths.
  exposed use ::former;
}

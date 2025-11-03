export let uiState =
{
  lightMode : "light",
  diamond : "white",
  metal : "silver",
  ring : 0,
  changed :
  [
    "lightMode",
    "diamond",
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

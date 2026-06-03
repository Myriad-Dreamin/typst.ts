
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that unparen with brackets stays as an LrElem.
#let item = $limits(sum)_i$
$
  1 / ([item]) quad
  1 /  [item]
$
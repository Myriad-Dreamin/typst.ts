
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Tests small tilings for pixel accuracy.
#box(
  width: 8pt,
  height: 1pt,
  fill: tiling(size: (1pt, 1pt), square(size: 1pt, fill: black))
)
#v(-1em)
#box(
  width: 8pt,
  height: 1pt,
  fill: tiling(size: (2pt, 1pt), square(size: 1pt, fill: black))
)
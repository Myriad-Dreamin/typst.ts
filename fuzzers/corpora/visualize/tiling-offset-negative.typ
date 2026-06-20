
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test with offset set to `(-10pt, -10pt)`
#let t(..args) = tiling(size: (30pt, 30pt), ..args)[
  #square(width: 100%, height: 100%, stroke: 1pt, fill: blue)
]

#set page(width: 100pt, height: 100pt)

#rect(fill: t(offset: (-10pt, -10pt)), width: 100%, height: 100%, stroke: 1pt)
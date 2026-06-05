
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test with spacing set to `(0pt, 0pt)`
#let t(..args) = tiling(size: (30pt, 30pt), ..args)[
  #square(width: 100%, height: 100%, stroke: 1pt, fill: blue)
]

#set page(width: 100pt, height: 100pt)

#rect(fill: t(spacing: (0pt, 0pt)), width: 100%, height: 100%, stroke: 1pt)
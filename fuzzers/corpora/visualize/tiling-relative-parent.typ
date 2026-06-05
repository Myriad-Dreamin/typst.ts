
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test with relative set to `"parent"`
#let t(fill, ..args) = tiling(size: (30pt, 30pt), ..args)[
  #rect(width: 100%, height: 100%, fill: fill, stroke: none)
  #place(top + left, line(start: (0%, 0%), end: (100%, 100%), stroke: 1pt))
  #place(top + left, line(start: (0%, 100%), end: (100%, 0%), stroke: 1pt))
]

#set page(fill: t(white), width: 100pt, height: 100pt)

#rect(fill: t(none, relative: "parent"), width: 100%, height: 100%, stroke: 1pt)
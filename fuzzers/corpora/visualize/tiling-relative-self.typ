
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test with relative set to `"self"`
#let t(..args) = tiling(size: (30pt, 30pt), ..args)[
  #set line(stroke: green)
  #place(top + left, line(start: (0%, 0%), end: (100%, 100%), stroke: 1pt))
  #place(top + left, line(start: (0%, 100%), end: (100%, 0%), stroke: 1pt))
]

#set page(fill: t(), width: 100pt, height: 100pt)
#rect(
  width: 100%,
  height: 100%,
  fill: t(relative: "self"),
  stroke: 1pt + green,
)
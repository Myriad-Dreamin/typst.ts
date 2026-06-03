
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test clipping with `outset`.
#set page(height: 60pt)

#box(
  outset: 5pt,
  stroke: 2pt + black,
  width: 20pt,
  height: 20pt,
  clip: true,
  image("/assets/images/rhino.png", width: 30pt)
)
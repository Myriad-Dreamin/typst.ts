
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test clipping with `radius`.

#set page(height: 60pt)

#box(
  radius: 5pt,
  stroke: 2pt + black,
  width: 20pt,
  height: 20pt,
  clip: true,
  image("/assets/files/rhino.png", width: 30pt)
)
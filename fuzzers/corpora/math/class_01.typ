
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test custom content.
#let dotsq = square(
  size: 0.7em,
  stroke: 0.5pt,
  align(center+horizon, circle(radius: 0.15em, fill: black))
)

$ a dotsq b \
  a class("normal", dotsq) b \
  a class("vary", dotsq) b \
  a + class("vary", dotsq) b \
  a class("punctuation", dotsq) b $

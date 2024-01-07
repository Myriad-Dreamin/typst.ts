
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test in HSL space.
#set page(
  width: 100pt,
  height: 100pt,
  fill: gradient.conic(red, purple, space: color.hsl)
)

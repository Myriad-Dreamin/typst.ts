
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The tests below test whether hue rotation works correctly.
// Here we test in Oklab space for reference.
#set page(
  width: 100pt,
  height: 30pt,
  fill: gradient.linear(red, purple, space: oklab)
)
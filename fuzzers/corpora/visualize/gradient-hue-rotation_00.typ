
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test in Oklab space for reference.
#set page(
  width: 100pt,
  height: 30pt,
  fill: gradient.linear(red, purple, space: oklab)
)

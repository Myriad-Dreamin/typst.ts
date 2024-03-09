
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that rotation impact layout.
#set page(width: 200pt)
#set rotate(reflow: true)

#let one(angle) = box(fill: aqua, rotate(angle)[Test Text])
#for angle in range(0, 360, step: 15) {
  one(angle * 1deg)
}

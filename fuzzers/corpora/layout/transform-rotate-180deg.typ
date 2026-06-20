
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that rotation impact layout.
#set page(width: 200pt)
#set rotate(reflow: true)

#let one(angle) = box(fill: aqua, rotate(angle)[Test Text\ Test Text])
#one(0deg)
#one(180deg)

- #one(0deg)
- #one(180deg)
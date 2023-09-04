
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test stroke composition.
#set square(stroke: 4pt)
#set text(font: "Roboto")
#square(
  stroke: (left: red, top: yellow, right: green, bottom: blue),
  radius: 100%, align(center+horizon)[*G*],
  inset: 8pt
)

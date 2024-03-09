
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test relative sizing in scaled boxes.
#set page(width: 200pt, height: 200pt)
#set text(size: 32pt)
#let scaled(body) = box(scale(
  x: 60%,
  y: 40%,
  box(stroke: 0.5pt, width: 30%, clip: true, body)
))

#set scale(reflow: false)
Hello #scaled[World]!\

#set scale(reflow: true)
Hello #scaled[World]!

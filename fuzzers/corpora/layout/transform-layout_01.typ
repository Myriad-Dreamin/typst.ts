
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test relative sizing in rotated boxes.
#set page(width: 200pt, height: 200pt)
#set text(size: 32pt)
#let rotated(body) = box(rotate(
  90deg,
  box(stroke: 0.5pt, height: 20%, clip: true, body)
))

#set rotate(reflow: false)
Hello #rotated[World]!\

#set rotate(reflow: true)
Hello #rotated[World]!

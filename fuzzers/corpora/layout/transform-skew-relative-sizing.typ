
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test relative sizing in skewed boxes.
#set page(width: 100pt, height: 60pt)
#set text(size: 12pt)
#let skewed(body) = box(skew(
  ax: 30deg,
  box(stroke: 0.5pt, width: 30%, clip: true, body)
))

#set skew(reflow: false)
Hello #skewed[World]!\

#set skew(reflow: true)
Hello #skewed[World]!
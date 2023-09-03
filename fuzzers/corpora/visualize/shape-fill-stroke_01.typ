
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test stroke folding.
#let sq(..args) = box(square(size: 10pt, ..args))

#set square(stroke: none)
#sq()
#set square(stroke: auto)
#sq()
#sq(fill: teal)
#sq(stroke: 2pt)
#sq(stroke: blue)
#sq(fill: teal, stroke: blue)
#sq(fill: teal, stroke: 2pt + blue)

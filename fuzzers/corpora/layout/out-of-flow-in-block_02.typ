
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Placed item in the first region.
#set page(height: 5cm, margin: 1cm)
Placed item in the first region.
#block(breakable: true, above: 1cm, stroke: 1pt, inset: 0.5cm)[
  #place(dx: -0.5cm, dy: -0.75cm, box(width: 200%)[OOF])
  #rect(height: 2cm, fill: gray)
]

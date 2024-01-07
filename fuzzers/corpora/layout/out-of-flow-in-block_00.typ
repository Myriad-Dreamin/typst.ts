
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// No item in the first region.
#set page(height: 5cm, margin: 1cm)
No item in the first region.
#block(breakable: true, stroke: 1pt, inset: 0.5cm)[
  #rect(height: 2cm, fill: gray)
]

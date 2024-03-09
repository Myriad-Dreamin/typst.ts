
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Counter update and placed item in the first region.
#set page(height: 5cm, margin: 1cm)
Counter update + place.
#block(breakable: true, above: 1cm, stroke: 1pt, inset: 0.5cm)[
  #counter("dummy").step()
  #place(dx: -0.5cm, dy: -0.75cm, box([OOF]))
  #rect(height: 2cm, fill: gray)
]

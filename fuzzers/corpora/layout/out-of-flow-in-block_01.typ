
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Counter update in the first region.
#set page(height: 5cm, margin: 1cm)
Counter update.
#block(breakable: true, stroke: 1pt, inset: 0.5cm)[
  #counter("dummy").step()
  #rect(height: 2cm, fill: gray)
]

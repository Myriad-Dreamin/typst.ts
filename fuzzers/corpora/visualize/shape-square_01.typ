
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test auto-sized square.
#square(fill: eastern)[
  #set text(fill: white, weight: "bold")
  Typst
]

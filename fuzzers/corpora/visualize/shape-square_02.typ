
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test relative-sized child.
#square(fill: eastern)[
  #rect(width: 10pt, height: 5pt, fill: conifer)
  #rect(width: 40%, height: 5pt, stroke: conifer)
]

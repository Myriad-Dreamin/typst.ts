
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test divider in a container.
#set page(width: 200pt)
#box(width: 150pt, stroke: 1pt, inset: 10pt)[
  Content before
  #divider()
  Content after
]
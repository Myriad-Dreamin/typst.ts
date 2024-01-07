
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// In-flow item with size zero in the first region.
#set page(height: 5cm, margin: 1cm)
In-flow, zero-sized item.
#block(breakable: true, stroke: 1pt, inset: 0.5cm)[
  #set block(spacing: 0pt)
  #line(length: 0pt)
  #rect(height: 2cm, fill: gray)
  #line(length: 100%)
]

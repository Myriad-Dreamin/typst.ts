
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test inset folding
#set grid(inset: 10pt)
#set grid(inset: (left: 0pt))

#grid(
  fill: red,
  inset: (right: 0pt),
  grid.cell(inset: (top: 0pt))[a]
)
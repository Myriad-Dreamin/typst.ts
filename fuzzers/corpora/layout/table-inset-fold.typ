
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test inset folding
#set table(inset: 10pt)
#set table(inset: (left: 0pt))

#table(
  fill: red,
  inset: (right: 0pt),
  table.cell(inset: (top: 0pt))[a]
)
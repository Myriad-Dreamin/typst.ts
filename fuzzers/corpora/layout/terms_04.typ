
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test grid like show rule.
#show terms: it => table(
  columns: 2,
  inset: 3pt,
  ..it.children.map(v => (emph(v.term), v.description)).flatten(),
)

/ A: One letter
/ BB: Two letters
/ CCC: Three letters

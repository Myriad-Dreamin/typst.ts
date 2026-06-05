
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that table cells aren't considered paragraphs by default.
#show par: highlight

#table(
  columns: 3,
  [A],
  block[B],
  par[C],
)

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Block below shouldn't expand to the end of the page, but stay within its
// rows' boundaries.
#set page(height: 9em)
#table(
  rows: (1em, 1em, 1fr, 1fr, auto),
  table.cell(rowspan: 2, block(width: 2em, height: 100%, fill: red)),
  table.cell(rowspan: 2, block(width: 2em, height: 100%, fill: red)),
  [a]
)

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Red line should be kept here
#set page(height: 6em)
#set text(6pt)
#table(
  column-gutter: 3pt,
  inset: 1pt,
  table.header(
    table.hline(stroke: red, position: bottom),
    [a],
  ),
  [a],
  table.cell(stroke: aqua)[b]
)
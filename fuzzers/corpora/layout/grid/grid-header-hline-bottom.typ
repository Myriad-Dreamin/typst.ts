
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Yellow line should be kept here
#set text(6pt)
#table(
  column-gutter: 3pt,
  inset: 1pt,
  table.header(
    [a],
    table.hline(stroke: yellow),
  ),
  table.cell(rowspan: 2)[b]
)
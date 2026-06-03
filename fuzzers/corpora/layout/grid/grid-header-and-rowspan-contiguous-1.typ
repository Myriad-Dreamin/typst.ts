
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Block should occupy all space
#set page(height: 15em)

#table(
  rows: (auto, 2.5em, 2em, auto),
  gutter: 3pt,
  inset: 0pt,
  table.header(
    [*H*],
    [*W*]
  ),
  table.cell(rowspan: 3, block(height: 2.5em + 2em + 20em, width: 100%, fill: red))
)
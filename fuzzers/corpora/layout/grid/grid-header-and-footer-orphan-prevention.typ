
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Orphan header prevention test
#set page(height: 13em)
#v(8em)
#grid(
  columns: 3,
  gutter: 5pt,
  grid.header(
    [*Mui*], [*A*], grid.cell(rowspan: 2, fill: orange)[*B*],
    [*Header*], [*Header* #v(0.1em)],
  ),
  ..([Test], [Test], [Test]) * 7,
  grid.footer(
    [*Mui*], [*A*], grid.cell(rowspan: 2, fill: orange)[*B*],
    [*Footer*], [*Footer* #v(0.1em)],
  ),
)
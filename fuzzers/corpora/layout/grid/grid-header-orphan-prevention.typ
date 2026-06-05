
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Orphan header prevention test
#set page(height: 12em)
#v(8em)
#grid(
  columns: 3,
  grid.header(
    [*Mui*], [*A*], grid.cell(rowspan: 2, fill: orange)[*B*],
    [*Header*], [*Header* #v(0.1em)]
  ),
  ..([Test], [Test], [Test]) * 20
)
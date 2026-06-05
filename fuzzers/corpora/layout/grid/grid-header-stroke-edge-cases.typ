
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test header stroke priority edge case (last header row removed)
#set page(height: 8em)
#table(
  columns: 2,
  stroke: black,
  gutter: (auto, 3pt),
  table.header(
    [c], [d],
  ),
  ..(table.cell(stroke: aqua)[d],) * 8,
)
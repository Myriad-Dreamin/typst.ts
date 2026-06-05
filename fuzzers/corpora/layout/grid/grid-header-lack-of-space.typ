
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test lack of space for header + text.
#set page(height: 8em)

#table(
  rows: (auto, 2.5em, auto, auto, 10em),
  gutter: 3pt,
  table.header(
    [*Hello*],
    [*World*]
  ),
  table.cell(rowspan: 3, lorem(80))
)
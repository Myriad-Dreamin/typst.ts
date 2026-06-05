
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// This should look right
#set page(height: 15em)

#table(
  rows: (auto, 2.5em, 2em, auto),
  gutter: 3pt,
  table.header(
    [*Hello*],
    [*World*]
  ),
  table.cell(rowspan: 3, lines(15))
)
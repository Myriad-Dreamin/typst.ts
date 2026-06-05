
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test lack of space for header + text.
#set page(height: 9em + 2.5em + 1.5em)

#table(
  rows: (auto, 2.5em, auto, auto, 10em, 2.5em, auto),
  gutter: 3pt,
  table.header[*Hello*][*World*],
  table.cell(rowspan: 3, lorem(30)),
  table.footer[*Ok*][*Bye*],
)

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Empty header should just be a repeated blank row
#set page(height: 12em)
#table(
  columns: 4,
  align: center + horizon,
  table.header(),
  ..range(0, 4).map(i => ([John \##i], table.cell(stroke: green)[123], table.cell(stroke: blue)[456], [789])).flatten()
)

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Headers
#set page(height: 15em)
#set text(dir: rtl)
#table(
  columns: 5,
  align: center + horizon,
  table.header(
    table.cell(colspan: 5)[*Cool Zone*],
    table.cell(stroke: red)[*N1*], table.cell(stroke: aqua)[*N2*], [*D1*], [*D2*], [*Etc*],
    table.hline(start: 2, end: 3, stroke: yellow)
  ),
  ..range(0, 10).map(i => ([\##i], table.cell(stroke: green)[123], table.cell(stroke: blue)[456], [789], [?], table.hline(start: 4, end: 5, stroke: red))).flatten()
)
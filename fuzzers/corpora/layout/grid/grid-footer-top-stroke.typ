
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Footer's top stroke should win when repeated, but lose at the last page.
#set page(height: 10em)
#table(
  stroke: green,
  table.header(table.cell(stroke: red)[Hello]),
  table.cell(stroke: yellow)[Hi],
  table.cell(stroke: yellow)[Bye],
  table.cell(stroke: yellow)[Ok],
  table.footer[Bye],
)
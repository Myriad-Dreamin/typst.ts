
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// When a footer has a rowspan with an empty row, it should be displayed
// properly
#set page(height: 14em, width: auto)

#let count = counter("g")
#table(
  rows: (auto, 2em, auto, auto),
  table.header(
    [eeec],
    table.cell(rowspan: 2, count.step() + context count.display()),
  ),
  [d],
  block(width: 5em, fill: yellow, lorem(7)),
  [d],
  table.footer(
    [eeec],
    table.cell(rowspan: 2, count.step() + context count.display()),
  )
)
#context count.display()
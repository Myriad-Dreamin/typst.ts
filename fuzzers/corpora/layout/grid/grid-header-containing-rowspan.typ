
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// When a header has a rowspan with an empty row, it should be displayed
// properly
#set page(height: 10em)

#let count = counter("g")
#table(
  rows: (auto, 2em, auto, auto),
  table.header(
    [eeec],
    table.cell(rowspan: 2, count.step() + context count.display()),
  ),
  [d],
  block(width: 5em, fill: yellow, lorem(15)),
  [d]
)
#context count.display()

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 6em)
#table(
  rows: (4em,) * 7 + (auto,) + (4em,) * 7,
  columns: 2,
  column-gutter: 1em,
  row-gutter: (1em, 2em) * 4,
  fill: (x, y) => if calc.odd(x + y) { orange.lighten(20%) } else { red },
  table.cell(rowspan: 15, [a \ ] * 15),
  [] * 15
)
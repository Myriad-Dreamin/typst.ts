
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 6em)
#table(
  rows: (3em,) * 15,
  columns: 2,
  column-gutter: 1em,
  row-gutter: (1em, 2em) * 4,
  fill: (x, y) => if calc.odd(x + y) { aqua } else { blue },
  table.cell(breakable: true, rowspan: 15, [a \ ] * 15),
  [] * 15
)

// Some splitting corner cases
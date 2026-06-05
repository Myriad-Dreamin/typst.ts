
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Fixed-size rows
#set page(height: 10em)
#grid(
  columns: 2,
  rows: 1.5em,
  fill: (x, y) => if calc.odd(x + y) { blue.lighten(50%) } else { blue.lighten(10%) },
  grid.cell(rowspan: 3)[R1], [b],
  [c],
  [d],
  [e], [f],
  grid.cell(rowspan: 5)[R2], [h],
  [i],
  [j],
  [k],
  [l],
  [m], [n]
)
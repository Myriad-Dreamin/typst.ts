
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test folding per-cell properties (align and inset)
#grid(
  columns: (1fr, 1fr),
  rows: (2.5em, auto),
  align: right,
  inset: 5pt,
  fill: (x, y) => (green, aqua).at(calc.rem(x + y, 2)),
  [Top], grid.cell(align: bottom)[Bot],
  grid.cell(inset: (bottom: 0pt))[Bot], grid.cell(inset: (bottom: 0pt))[Bot]
)
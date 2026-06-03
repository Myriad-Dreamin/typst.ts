
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test folding per-cell properties (align and inset)
#table(
  columns: (1fr, 1fr),
  rows: (2.5em, auto),
  align: right,
  fill: (x, y) => (green, aqua).at(calc.rem(x + y, 2)),
  [Top], table.cell(align: bottom)[Bot],
  table.cell(inset: (bottom: 0pt))[Bot], table.cell(inset: (bottom: 0pt))[Bot]
)
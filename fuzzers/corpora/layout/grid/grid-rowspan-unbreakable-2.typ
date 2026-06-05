
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test cell breakability
#show grid.cell: it => {
  test(it.breakable, (it.x, it.y) != (0, 6) and (it.y in (2, 5, 6) or (it.x, it.y) in ((0, 1), (2, 3), (1, 7))))
  it.breakable
}
#grid(
  columns: 3,
  rows: (6pt, 1fr, auto, 1%, 1em, auto, auto, 0.2in),
  row-gutter: (0pt, 0pt, 0pt, auto),
  [a], [b], [c],
  grid.cell(rowspan: 3)[d], [e], [f],
  [g], [h],
  [i], grid.cell(rowspan: 2)[j],
  [k],
  grid.cell(y: 5)[l],
  grid.cell(y: 6, breakable: false)[m], grid.cell(y: 6, breakable: true)[n],
  grid.cell(y: 7, breakable: false)[o], grid.cell(y: 7, breakable: true)[p], grid.cell(y: 7, breakable: auto)[q]
)

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Red line should be above [c] (hline skips the shortest rowspan).
#set text(6pt)
#table(
  rows: 1em,
  columns: 2,
  inset: 1.5pt,
  table.cell(rowspan: 3)[a], table.cell(rowspan: 2)[b],
  table.hline(stroke: red),
  [c]
)
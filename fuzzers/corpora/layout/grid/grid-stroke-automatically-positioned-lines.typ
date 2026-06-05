
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Automatically positioned lines
// Plus stroke thickness ordering
#table(
  columns: 3,
  table.hline(stroke: red + 5pt),
  table.vline(stroke: blue + 5pt),
  table.vline(stroke: 2pt),
  [a],
  table.vline(x: 1, stroke: aqua + 5pt),
  [b],
  table.vline(stroke: aqua + 5pt),
  [c],
  table.vline(stroke: yellow + 5.2pt),
  table.hline(stroke: green + 5pt),
  [a], [b], [c],
  [a], table.hline(stroke: green + 2pt), table.vline(stroke: 2pt), [b], [c],
)
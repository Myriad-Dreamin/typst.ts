
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test overriding outside alignment
#set align(bottom + right)
#grid(
  columns: (1fr, 1fr),
  rows: 2em,
  align: auto,
  fill: green,
  [BR], [BR],
  grid.cell(align: left, fill: aqua)[BL], grid.cell(align: top, fill: red.lighten(50%))[TR]
)
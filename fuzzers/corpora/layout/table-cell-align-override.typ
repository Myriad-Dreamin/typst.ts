
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test overriding outside alignment
#set align(bottom + right)
#table(
  columns: (1fr, 1fr),
  rows: 2em,
  align: auto,
  fill: green,
  [BR], [BR],
  table.cell(align: left, fill: aqua)[BL], table.cell(align: top, fill: red.lighten(50%))[TR]
)
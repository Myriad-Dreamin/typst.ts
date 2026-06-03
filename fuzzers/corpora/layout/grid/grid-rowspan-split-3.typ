
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 5em)
#table(
  columns: 2,
  fill: red,
  inset: 0pt,
  table.cell(fill: orange, rowspan: 10, place(bottom)[*Z*] + [x\ ] * 10 + place(bottom)[*ZZ*]),
  ..([y],) * 10,
  [a], [b],
)
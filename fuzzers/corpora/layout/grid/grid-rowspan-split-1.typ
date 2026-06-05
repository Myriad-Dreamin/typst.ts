
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 10em)
#table(
  columns: 2,
  rows: (auto, auto, 3em),
  fill: red,
  [a], table.cell(rowspan: 3, block(width: 50%, height: 10em, fill: orange) + place(bottom)[*ZD*]),
  [e],
  [f]
)
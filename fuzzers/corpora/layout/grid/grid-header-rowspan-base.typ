
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 7em)
#set text(6pt)
#let full-block = block(width: 2em, height: 100%, fill: red)
#table(
  columns: 3,
  inset: 1.5pt,
  table.header(
    [a], full-block, table.cell(rowspan: 2, full-block),
    [b]
  )
)
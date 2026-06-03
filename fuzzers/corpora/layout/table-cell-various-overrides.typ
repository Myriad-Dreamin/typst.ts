
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#table(
  columns: 2,
  fill: green,
  align: right,
  [*Name*], [*Data*],
  table.cell(fill: blue)[J.], [Organizer],
  table.cell(align: center)[K.], [Leader],
  [M.], table.cell(inset: 0pt)[Player]
)
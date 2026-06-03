
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(margin: (left: 1.5cm))
#set par.line(numbering: "1", number-clearance: 0.5cm)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 0.5cm,
  stroke: 0.5pt,

  grid.cell(rowspan: 2)[very #box(fill: red, height: 4cm)[tall]],
  grid.cell(inset: (y: 0.5pt))[Line 1\ Line 2\ Line 3],
  grid.cell(inset: (y: 0.5pt))[Line 4\ Line 5\ Line 6\ Line 7\ Line 8\ Line 9\ End]
)
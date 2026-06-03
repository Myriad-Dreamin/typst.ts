
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#table(
  columns: 3,
  [a], table.cell(colspan: 2)[b c],
  table.cell(stroke: blue)[d], [e], [f],
  [g], [h], table.cell(stroke: (left: yellow, top: green, right: aqua, bottom: red))[i],
  [j], [k], [l],
  table.cell(stroke: 3pt)[m], [n], table.cell(stroke: (dash: "loosely-dotted"))[o],
)
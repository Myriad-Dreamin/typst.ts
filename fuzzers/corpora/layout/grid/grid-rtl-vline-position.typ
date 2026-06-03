
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test left and right for vlines in RTL
#set text(dir: rtl)
#grid(
  columns: 3,
  inset: 5pt,
  grid.vline(stroke: red, position: left), grid.vline(stroke: green, position: right), [a],
  grid.vline(stroke: red, position: left), grid.vline(stroke: 2pt, position: right), [b],
  grid.vline(stroke: red, position: left), grid.vline(stroke: 2pt, position: right), [c],
  grid.vline(stroke: aqua, position: right)
)

#grid(
  columns: 3,
  inset: 5pt,
  gutter: 3pt,
  grid.vline(stroke: green, position: left), grid.vline(stroke: red, position: right), [a],
  grid.vline(stroke: blue, position: left), grid.vline(stroke: red, position: right), [b],
  grid.vline(stroke: blue, position: left), grid.vline(stroke: red, position: right), [c],
  grid.vline(stroke: 2pt, position: right)
)

#grid(
  columns: 3,
  inset: 5pt,
  grid.vline(stroke: green, position: start), grid.vline(stroke: red, position: end), [a],
  grid.vline(stroke: 2pt, position: start), grid.vline(stroke: red, position: end), [b],
  grid.vline(stroke: 2pt, position: start), grid.vline(stroke: red, position: end), [c],
  grid.vline(stroke: 2pt, position: start)
)

#grid(
  columns: 3,
  inset: 5pt,
  gutter: 3pt,
  grid.vline(stroke: green, position: start), grid.vline(stroke: red, position: end), [a],
  grid.vline(stroke: blue, position: start), grid.vline(stroke: red, position: end), [b],
  grid.vline(stroke: blue, position: start), grid.vline(stroke: red, position: end), [c],
  grid.vline(stroke: 2pt, position: start)
)
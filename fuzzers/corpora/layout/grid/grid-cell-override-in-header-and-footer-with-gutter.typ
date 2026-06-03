
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#table(
  gutter: 3pt,
  table.header(table.cell(stroke: red)[Hello]),
  table.footer(table.cell(stroke: aqua)[Bye]),
)
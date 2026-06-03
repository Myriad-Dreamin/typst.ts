
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#table(
  columns: 3,
  [Outside],
  table.header(
    [A], table.cell(x: 1)[B], [C],
    table.cell(x: 1)[D],
  ),
)
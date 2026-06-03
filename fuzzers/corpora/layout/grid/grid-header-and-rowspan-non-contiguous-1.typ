
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Rowspan sizing algorithm doesn't do the best job at non-contiguous content
// ATM.
#set page(height: 15em)

#table(
  rows: (auto, 2.5em, 2em, auto, 5em),
  table.header(
    [*Hello*],
    [*World*]
  ),
  table.cell(rowspan: 3, lines(15))
)
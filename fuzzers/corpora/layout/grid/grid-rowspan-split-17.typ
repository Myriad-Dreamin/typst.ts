
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#show table.cell.where(x: 0): strong
#show table.cell.where(y: 0): strong
#set page(height: 13em)
#let lets-repeat(thing, n) = ((thing + colbreak(),) * (calc.max(0, n - 1)) + (thing,)).join()
#table(
  columns: 4,
  fill: (x, y) => if x == 0 or y == 0 { gray },
  [], [Test 1], [Test 2], [Test 3],
  table.cell(rowspan: 15, align: horizon, lets-repeat((rotate(-90deg, reflow: true)[*All Tests*]), 3)),
  ..([123], [456], [789]) * 15
)
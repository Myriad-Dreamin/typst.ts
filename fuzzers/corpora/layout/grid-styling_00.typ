
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(height: 70pt)
#set grid(fill: (x, y) => if calc.even(x + y) { rgb("aaa") })

#grid(
  columns: (1fr,) * 3,
  stroke: 2pt + rgb("333"),
  [A], [B], [C], [], [], [D \ E \ F \ \ \ G], [H],
)

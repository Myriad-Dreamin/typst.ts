
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(margin: (left: 1.5cm))
#set par.line(numbering: "1", number-clearance: 0.5cm)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 0.5cm,
  inset: 5pt,
  block[A\ #box(lorem(5))], [Roses\ are\ red],
  [AAA], [],
  [], block[BBB\ CCC],
)
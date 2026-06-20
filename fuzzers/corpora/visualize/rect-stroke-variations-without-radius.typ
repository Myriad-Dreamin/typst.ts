
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "block.typ": test-block
#set page(width: 7.5cm, margin: 0pt)
#table(
  stroke: none,
  columns: (0.75fr,) + 3 * (1fr,),
  [], [butt], [square], [round],
  [no dash],
  test-block(cap: "butt"),
  test-block(cap: "square"),
  test-block(cap: "round"),

  [dashed],
  test-block(cap: "butt", dash: "dashed"),
  test-block(cap: "square", dash: "dashed"),
  test-block(cap: "round", dash: "dashed"),

  [loosely-dashed],
  test-block(cap: "butt", dash: "loosely-dashed"),
  test-block(cap: "square", dash: "loosely-dashed"),
  test-block(cap: "round", dash: "loosely-dashed"),
)
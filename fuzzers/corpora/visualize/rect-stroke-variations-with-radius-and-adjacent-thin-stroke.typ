
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "block.typ": test-block
#set page(width: 7.5cm, margin: 0pt)
#table(
  stroke: none,
  columns: (0.75fr,) + 3 * (1fr,),
  [], [butt], [square], [round],
  [no dash],
  test-block(cap: "butt", radius: 12pt, adjacent: 1pt),
  test-block(cap: "square", radius: 12pt, adjacent: 1pt),
  test-block(cap: "round", radius: 12pt, adjacent: 1pt),

  [dashed],
  test-block(cap: "butt", radius: 12pt, adjacent: 1pt, dash: "dashed"),
  test-block(cap: "square", radius: 12pt, adjacent: 1pt, dash: "dashed"),
  test-block(cap: "round", radius: 12pt, adjacent: 1pt, dash: "dashed"),

  [loosely-dashed],
  test-block(cap: "butt", radius: 12pt, adjacent: 1pt, dash: "loosely-dashed"),
  test-block(cap: "square", radius: 12pt, adjacent: 1pt, dash: "loosely-dashed"),
  test-block(cap: "round", radius: 12pt, adjacent: 1pt, dash: "loosely-dashed"),
)
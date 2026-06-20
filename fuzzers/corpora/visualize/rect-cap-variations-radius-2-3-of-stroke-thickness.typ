
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "block.typ": another-block
#set page(width: 7.5cm, margin: 0pt)
#table(
  stroke: none,
  columns: (0.75fr,) + 3 * (1fr,),
  [], [none], [0 width], [thin],
  [butt],
  another-block(cap: "butt", radius: 4pt, adjacent: none),
  another-block(cap: "butt", radius: 4pt, adjacent: 0pt),
  another-block(cap: "butt", radius: 4pt, adjacent: 1pt),

  [square],
  another-block(cap: "square", radius: 4pt, adjacent: none),
  another-block(cap: "square", radius: 4pt, adjacent: 0pt),
  another-block(cap: "square", radius: 4pt, adjacent: 1pt),

  [round],
  another-block(cap: "round", radius: 4pt, adjacent: none),
  another-block(cap: "round", radius: 4pt, adjacent: 0pt),
  another-block(cap: "round", radius: 4pt, adjacent: 1pt),
)
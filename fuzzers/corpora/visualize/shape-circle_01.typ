
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test auto sizing.
#set circle(inset: 0pt)

Auto-sized circle.
#circle(fill: rgb("eb5278"), stroke: 2pt + black,
  align(center + horizon)[But, soft!]
)

Center-aligned rect in auto-sized circle.
#circle(fill: forest, stroke: conifer,
  align(center + horizon,
    rect(fill: conifer, inset: 5pt)[But, soft!]
  )
)

Rect in auto-sized circle.
#circle(fill: forest,
  rect(fill: conifer, stroke: white, inset: 4pt)[
    #set text(8pt)
    But, soft! what light through yonder window breaks?
  ]
)

Expanded by height.
#circle(stroke: black, align(center)[A \ B \ C])

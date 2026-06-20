
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let rects = grid(columns: (7mm, 7mm, 7mm, 7mm), gutter: 5pt,
  rect(), rect(width: -7mm), rect(height: -1cm), rect(width: -7mm, height: -1cm)
)
#set rect(fill: red, width: 7mm, height: 1cm, radius: 40%)
#set align(center + horizon)
#rects
#set rect(stroke: black + 3pt)
#rects
#set rect(stroke: (left: black + 3pt, top: blue + 3pt, right: green + 3pt, bottom: yellow + 3pt))
#rects
#set rect(radius: 0pt)
#rects
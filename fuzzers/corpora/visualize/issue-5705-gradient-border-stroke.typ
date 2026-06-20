
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let stroke = (
  top: gradient.linear(green, black, yellow, angle: 90deg) + 15pt,
  right: gradient.linear(green, black, yellow, angle: 180deg) + 15pt,
  bottom: gradient.linear(green, black, yellow, angle: 270deg) + 15pt,
  left: gradient.linear(green, black, yellow, angle: 0deg) + 15pt,
)
#set align(center + horizon)
#rect(width: 100pt, height: 100pt, radius: 15pt, stroke: stroke,
  rect(width: 65pt, height: 65pt, radius: 0pt, stroke: stroke,
    rect(width: 30pt, height: 30pt, radius: 30pt, stroke: stroke)
  )
)
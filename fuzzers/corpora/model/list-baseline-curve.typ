
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let dy = 15pt
- #curve(
    stroke: 5pt,
    curve.move((0pt,  30pt + dy)),
    curve.line((30pt, 30pt + dy)),
    curve.line((15pt, dy)),
    curve.close()
  )
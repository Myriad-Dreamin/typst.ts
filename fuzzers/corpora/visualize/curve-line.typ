
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#curve(
  fill: purple,
  stroke: 3pt + purple.lighten(50%),
  curve.move((0pt, 0pt)),
  curve.line((30pt, 30pt)),
  curve.line((0pt, 30pt)),
  curve.line((30pt, 0pt)),
)
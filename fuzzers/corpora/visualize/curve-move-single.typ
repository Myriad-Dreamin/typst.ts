
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#curve(
  stroke: 5pt,
  curve.move((0pt,  30pt)),
  curve.line((30pt, 30pt)),
  curve.line((15pt, 0pt)),
  curve.close()
)
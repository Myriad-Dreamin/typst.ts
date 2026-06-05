
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#curve(
  stroke: 2pt,
  curve.line((20pt, 0pt)),
  curve.move((0pt,  10pt)),
  curve.line((20pt, 10pt)),
  curve.move((0pt,  20pt)),
  curve.line((20pt, 20pt)),
)
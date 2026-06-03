
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#curve(
  fill: yellow,
  stroke: yellow.darken(20%),
  curve.move((10pt, 10pt)),
  curve.line((20pt, 10pt)),
  curve.line((20pt, 20pt)),
  curve.close(),
  curve.move((0pt, 5pt)),
  curve.line((25pt, 5pt)),
  curve.line((25pt, 30pt)),
  curve.close(mode: "smooth"),
)
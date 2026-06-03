
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#stack(
  dir: ltr,
  curve(
  fill: red,
  fill-rule: "non-zero",
  curve.move((25pt, 0pt)),
  curve.line((10pt, 50pt)),
  curve.line((50pt, 20pt)),
  curve.line((0pt, 20pt)),
  curve.line((40pt, 50pt)),
  curve.close()
  ),
  curve(
    fill: red,
    fill-rule: "even-odd",
    curve.move((25pt, 0pt)),
    curve.line((10pt, 50pt)),
    curve.line((50pt, 20pt)),
    curve.line((0pt, 20pt)),
    curve.line((40pt, 50pt)),
    curve.close()
  )
)
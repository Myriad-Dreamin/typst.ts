
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Curves/Paths used to implement `LayoutMultiple` rather than `LayoutSingle`
// without fulfilling the necessary contract of respecting region expansion.
#block(
  fill: aqua,
  width: 20pt,
  height: 15pt,
  curve(
    curve.move((0pt, 0pt)),
    curve.line((10pt, 10pt)),
  ),
)
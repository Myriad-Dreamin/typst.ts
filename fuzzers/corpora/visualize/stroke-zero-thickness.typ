
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 0pt strokes must function exactly like 'none' strokes and not draw anything
#rect(width: 10pt, height: 10pt, stroke: none)
#rect(width: 10pt, height: 10pt, stroke: 0pt)
#rect(width: 10pt, height: 10pt, stroke: none, fill: blue)
#rect(width: 10pt, height: 10pt, stroke: 0pt + red, fill: blue)

#line(length: 30pt, stroke: 0pt)
#line(length: 30pt, stroke: (paint: red, thickness: 0pt, dash: ("dot", 1pt)))

#table(columns: 2, stroke: none)[A][B]
#table(columns: 2, stroke: 0pt)[A][B]

#curve(
  stroke: none,
  curve.move((0pt,  30pt)),
  curve.line((30pt, 30pt)),
  curve.line((15pt, 0pt)),
  curve.close()
)

#curve(
  stroke: 0pt,
  curve.move((0pt,  30pt)),
  curve.line((30pt, 30pt)),
  curve.line((15pt, 0pt)),
  curve.close()
)
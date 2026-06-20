
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)
#let stroke = 10pt + gradient.linear(green, yellow, blue).sharp(5)
#let fill = gradient.linear(lime, yellow.lighten(60%), aqua).sharp(5)
#let scale = gradient.linear(black, white, black, white, black).sharp(5)
#let marked = (shape) => {
  shape
  place(center + top, line(length: 60pt, stroke: scale))
  place(center + horizon, line(length: 50pt, stroke: scale))
}
#grid(columns: 2, gutter: 15pt,
  marked(rect(width: 50pt, height: 50pt, radius: 0pt, stroke: stroke, fill: fill)),
  marked(rect(width: 50pt, height: 50pt, radius: 20pt, stroke: stroke, fill: fill)),
  marked(curve(stroke: stroke, fill: fill, curve.line((50pt, 0pt)), curve.line((50pt, 50pt)), curve.line((0pt, 50pt)), curve.close())),
  marked(circle(radius: 25pt, stroke: stroke, fill: fill)),
)
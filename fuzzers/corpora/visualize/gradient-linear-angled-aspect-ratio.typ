
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let grad = gradient.linear(angle: 135deg, ..color.map.inferno).sharp(6)
#grid(
  columns: 2,
  gutter: 5pt,
  rect(width: 70pt, height: 70pt, fill: grad),
  rect(width: 25pt, height: 70pt, fill: grad),
  rect(width: 70pt, height: 25pt, fill: grad),
)
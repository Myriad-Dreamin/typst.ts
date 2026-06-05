
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#curve(
  fill: blue.lighten(80%),
  stroke: blue,
  curve.move((0pt, 40pt)),
  curve.cubic((0pt, 70pt), (10pt, 80pt), (40pt, 80pt)),
  curve.cubic(auto, (80pt, 70pt), (80pt, 40pt)),
  curve.cubic(auto, (70pt, 0pt), (40pt, 0pt)),
  curve.close(mode: "straight")
)
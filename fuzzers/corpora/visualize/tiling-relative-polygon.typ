
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#polygon(
  fill: tiling(relative: "parent", circle(radius: 10pt)),
  stroke: blue,
  (20%, 0pt),
  (60%, 0pt),
  (80%, 20pt),
  (0%,  20pt),
)
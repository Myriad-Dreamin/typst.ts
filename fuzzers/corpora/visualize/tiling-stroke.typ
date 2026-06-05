
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test tiling on strokes
#align(
  center + top,
  square(
    size: 50pt,
    fill: tiling(
      size: (5pt, 5pt),
      align(horizon + center, circle(fill: blue, radius: 2.5pt))
    ),
    stroke: 7.5pt + tiling(
      size: (5pt, 5pt),
      align(horizon + center, circle(fill: red, radius: 2.5pt))
    )
  )
)
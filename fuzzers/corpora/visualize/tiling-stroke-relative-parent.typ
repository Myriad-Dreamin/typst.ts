
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test tiling on strokes with relative set to `"parent"`
// The tiling on the circle should align with the tiling on the square.
#align(
  center + top,
  block(
    width: 50pt,
    height: 50pt,
    fill: tiling(size: (5pt, 5pt), circle(radius: 2.5pt, fill: blue)),
    align(center + horizon, circle(
      radius: 15pt,
      stroke: 7.5pt + tiling(
        size: (5pt, 5pt), circle(radius: 2.5pt, fill: red), relative: "parent"
      ),
    ))
  )
)
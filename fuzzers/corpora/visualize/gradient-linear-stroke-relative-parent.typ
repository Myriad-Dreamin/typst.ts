
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The image should look as if there is a single gradient that is being used for
// both the circle stroke and the block fill.
#align(
  center + horizon,
  block(
    width: 50pt,
    height: 50pt,
    fill: gradient.linear(red, blue).sharp(4),
    circle(
      radius: 18pt,
      stroke: 5pt + gradient.linear(red, blue, relative: "parent").sharp(4),
    )
  )
)
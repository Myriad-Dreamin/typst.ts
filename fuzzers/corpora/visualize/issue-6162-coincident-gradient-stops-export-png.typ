
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that multiple gradient stops with the same position
// don't cause a panic.
#rect(
  fill: gradient.linear(
    (red, 0%),
    (green, 0%),
    (blue, 100%),
  )
)
#rect(
  fill: gradient.linear(
    (red, 0%),
    (green, 100%),
    (blue, 100%),
  )
)
#rect(
  fill: gradient.linear(
    (white, 0%),
    (red, 50%),
    (green, 50%),
    (blue, 100%),
  )
)

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(
  gradient.linear(red, green, blue, space: rgb).repeat(2).stops(),
  ((red, 0%), (green, 25%), (blue, 50%), (red, 50%), (green, 75%), (blue, 100%))
)
#test(
  gradient.linear(red, green, blue, space: rgb).repeat(2, mirror: true).stops(),
  ((red, 0%), (green, 25%), (blue, 50%), (green, 75%), (red, 100%))
)
// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(
  gradient.linear(red, green, blue).repeat(2).stops(),
  ((red, 0%), (green, 25%), (blue, 50%), (red, 50%), (green, 75%), (blue, 100%))
)
#test(
  gradient.linear(red, green, blue).repeat(2, mirror: true).stops(),
  ((red, 0%), (green, 25%), (blue, 50%), (green, 75%), (red, 100%))
)

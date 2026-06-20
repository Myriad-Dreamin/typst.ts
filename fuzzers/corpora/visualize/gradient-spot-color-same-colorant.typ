
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test gradient with spot colors of the same colorant (should interpolate tint)
#let pantone = color.spot("PANTONE 2221 C", eastern)
#set page(width: 100pt, height: 30pt, margin: 0pt)
#let g = gradient.linear(
  pantone.tint(20%),
  pantone.tint(100%),
  space: pantone,
)
#test(g.sample(10%).space(), pantone)
#block(
  width: 100%,
  height: 100%,
  fill: g,
)
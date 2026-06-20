
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test gradient with spot color mixed with process color (uses fallback in process space)
#let pantone = color.spot("PANTONE 185 C", rgb(89.4%, 0.7%, 17%))
#set page(width: 100pt, height: 30pt, margin: 0pt)
#block(
  width: 100%,
  height: 100%,
  fill: gradient.linear(
    pantone.tint(80%),
    blue,
    space: rgb,
  ),
)
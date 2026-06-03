
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 100pt)
#set align(center)

#lines(3)
#block(width: 80%, height: 60pt, fill: aqua)
#lines(2)
#block(
  breakable: false,
  width: 100%,
  inset: 4pt,
  fill: aqua,
  lines(3) + colbreak(),
)
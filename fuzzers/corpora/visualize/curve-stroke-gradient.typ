
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)
#let down = curve.line((40pt, 40pt), relative: true)
#let up = curve.line((40pt, -40pt), relative: true)

#curve(
  stroke: 4pt + gradient.linear(red, blue),
  down, up, down, up, down,
)
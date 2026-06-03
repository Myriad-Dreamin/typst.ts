
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test CMYK color conversion.
#let c = cmyk(50%, 64%, 16%, 17%)
#stack(
  dir: ltr,
  spacing: 1fr,
  rect(width: 1cm, fill: cmyk(69%, 11%, 69%, 41%)),
  rect(width: 1cm, fill: c),
  rect(width: 1cm, fill: c.negate(space: cmyk)),
)

#for x in range(0, 11) {
  box(square(size: 9pt, fill: c.lighten(x * 10%)))
}
#for x in range(0, 11) {
  box(square(size: 9pt, fill: c.darken(x * 10%)))
}
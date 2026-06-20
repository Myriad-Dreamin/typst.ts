
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test saturation.
#let col = color.hsl(180deg, 0%, 50%)
#for x in range(0, 11) {
  box(square(size: 9pt, fill: col.saturate(x * 10%)))
}

#let col = color.hsl(180deg, 100%, 50%)
#for x in range(0, 11) {
  box(square(size: 9pt, fill: col.desaturate(x * 10%)))
}

#let col = color.hsv(180deg, 0%, 50%)
#for x in range(0, 11) {
  box(square(size: 9pt, fill: col.saturate(x * 10%)))
}

#let col = color.hsv(180deg, 100%, 50%)
#for x in range(0, 11) {
  box(square(size: 9pt, fill: col.desaturate(x * 10%)))
}

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test skewing along both axes.
#set page(width: 100pt, height: 250pt)
#set text(size: 12pt)
#let skewed(angle) = box(skew(ax: 30deg, ay: angle)[Some Text])

#set skew(reflow: true)
#for angle in range(-30, 31, step: 10) {
  skewed(angle * 1deg)
}
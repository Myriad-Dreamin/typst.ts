
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that minimum wins if both width and height are given.
#stack(
  dir: ltr,
  spacing: 2pt,
  square(width: 20pt, height: 40pt),
  circle(width: 20%, height: 100pt),
)

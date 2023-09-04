
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test different ways of sizing.
#set page(width: 120pt, height: 40pt)
#stack(
  dir: ltr,
  spacing: 2pt,
  circle(radius: 5pt),
  circle(width: 10%),
  circle(height: 50%),
)

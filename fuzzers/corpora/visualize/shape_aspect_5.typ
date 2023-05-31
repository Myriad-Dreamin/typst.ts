// Test that squares and circles respect their 1-1 aspect ratio.

// Test different ways of sizing.
#set page(width: 120pt, height: 40pt, margin: 10pt)
#stack(
  dir: ltr,
  spacing: 2pt,
  circle(radius: 5pt),
  circle(width: 10%),
  circle(height: 50%),
)
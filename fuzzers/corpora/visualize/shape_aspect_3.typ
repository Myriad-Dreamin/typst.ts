// Test that squares and circles respect their 1-1 aspect ratio.

// Test that minimum wins if both width and height are given.
#set page(width: 120pt, height: 40pt, margin: 10pt)
#stack(
  dir: ltr,
  spacing: 2pt,
  square(width: 20pt, height: 40pt),
  circle(width: 20%, height: 100pt),
)

// Test the `circle` function.

#set page(width: 120pt, height: auto, margin: 10pt)

#let conifer = rgb("#9feb52")
#let forest = rgb("#43a127")

// Ensure circle directly in rect works.
#rect(width: 40pt, height: 30pt, fill: forest,
  circle(fill: conifer))

// Test that squares and circles respect their 1-1 aspect ratio.

#let conifer = rgb("#9feb52")
#let forest = rgb("#43a127")

// Test square that is limited by region size.
#set page(width: 20pt, height: 10pt, margin: 0pt)
#stack(dir: ltr, square(fill: forest), square(fill: conifer))

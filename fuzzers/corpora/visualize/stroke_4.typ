// Test lines.

#set page(width: 120pt, height: auto, margin: 10pt)

// Dashing
#line(length: 60pt, stroke: (paint: red, thickness: 1pt, dash: ("dot", 1pt)))
#v(3pt)
#line(length: 60pt, stroke: (paint: red, thickness: 1pt, dash: ("dot", 1pt, 4pt, 2pt)))
#v(3pt)
#line(length: 60pt, stroke: (paint: red, thickness: 1pt, dash: (array: ("dot", 1pt, 4pt, 2pt), phase: 5pt)))
#v(3pt)
#line(length: 60pt, stroke: (paint: red, thickness: 1pt, dash: ()))
#v(3pt)
#line(length: 60pt, stroke: (paint: red, thickness: 1pt, dash: (1pt, 3pt, 9pt)))

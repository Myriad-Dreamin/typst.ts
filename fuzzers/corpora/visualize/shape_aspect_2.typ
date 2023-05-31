// Test that squares and circles respect their 1-1 aspect ratio.
// Test alignment in automatically sized square and circle.
#set text(8pt)
#set page(width: auto, height: auto, margin: 10pt)
#box(square(inset: 4pt)[
  Hey there, #align(center + bottom, rotate(180deg, [you!]))
])
#box(circle(align(center + horizon, [Hey.])))
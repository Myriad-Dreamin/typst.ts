
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test alignment in automatically sized square and circle.
#set text(8pt)
#box(square(inset: 4pt)[
  Hey there, #align(center + bottom, rotate(180deg, [you!]))
])
#box(circle(align(center + horizon, [Hey.])))

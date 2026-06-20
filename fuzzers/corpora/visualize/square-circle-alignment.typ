
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test alignment in automatically sized square and circle.
#set text(8pt)
#stack(
  dir: ltr,
  spacing: 0.5em,
  square(inset: 4pt)[
    Hey there, #align(center + bottom, rotate(180deg, [you!]))
  ],
  circle(align(center + horizon, [Hey.]))
)

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that even spaces across multiple layout items are trimmed during
// line breaking.
#block(width: 15pt, box(fill: aqua, underline("A   " + text(fill: blue, " ") + "    B")))
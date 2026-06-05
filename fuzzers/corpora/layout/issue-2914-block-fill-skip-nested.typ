
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that fill and stroke are skipped for an empty frame with a nested block.
#set page(height: 50pt)
A
#block(fill: aqua, stroke: blue, inset: 5pt, width: 100%, block[B])
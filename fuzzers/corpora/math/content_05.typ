
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test boxes with a baseline are respected
#box(stroke: 0.2pt, $a #box(baseline:0.5em, stroke: 0.2pt, $a$)$)

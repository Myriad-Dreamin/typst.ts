
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test boxes without a baseline act as if the baseline is at the base
#{
     box(stroke: 0.2pt, $a #box(stroke: 0.2pt, $a$)$)
     h(12pt)
     box(stroke: 0.2pt, $a #box(stroke: 0.2pt, $g$)$)
     h(12pt)
     box(stroke: 0.2pt, $g #box(stroke: 0.2pt, $g$)$)
}

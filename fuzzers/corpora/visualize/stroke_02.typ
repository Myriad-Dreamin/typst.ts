
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Rectangle strokes
#rect(width: 20pt, height: 20pt, stroke: red)
#v(3pt)
#rect(width: 20pt, height: 20pt, stroke: (rest: red, top: (paint: blue, dash: "dashed")))
#v(3pt)
#rect(width: 20pt, height: 20pt, stroke: (thickness: 5pt, join: "round"))

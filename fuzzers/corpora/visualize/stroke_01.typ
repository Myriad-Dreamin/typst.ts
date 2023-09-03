
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Set rules with stroke
#set line(stroke: (paint: red, thickness: 1pt, cap: "butt", dash: "dash-dotted"))
#line(length: 60pt)
#v(3pt)
#line(length: 60pt, stroke: blue)
#v(3pt)
#line(length: 60pt, stroke: (dash: none))

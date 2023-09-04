
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Some simple test lines
#line(length: 60pt, stroke: red)
#v(3pt)
#line(length: 60pt, stroke: 2pt)
#v(3pt)
#line(length: 60pt, stroke: blue + 1.5pt)
#v(3pt)
#line(length: 60pt, stroke: (paint: red, thickness: 1pt, dash: "dashed"))
#v(3pt)
#line(length: 60pt, stroke: (paint: red, thickness: 4pt, cap: "round"))

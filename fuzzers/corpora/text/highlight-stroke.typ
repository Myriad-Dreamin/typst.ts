
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test highlight stroke
#highlight(stroke: 2pt + blue)[abc]
#highlight(stroke: (top: blue, left: red, bottom: green, right: orange))[abc]
#highlight(stroke: 1pt, radius: 3pt)[#lorem(5)]
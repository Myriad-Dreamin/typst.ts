
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 100pt, columns: 2)
#set place(float: true, scope: "parent", clearance: 10pt)
#let t(align, fill) = place(top + align, rect(fill: fill, height: 25pt))

#t(left, aqua)
#t(center, forest)
#t(right, conifer)
#lines(7)
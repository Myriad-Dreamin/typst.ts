
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto, height: auto)
#set place(float: true, clearance: 5pt)

#place(bottom, rect(width: 80pt, height: 10pt))
#place(top + center, rect(height: 20pt))
#align(center)[A]
#pagebreak()
#align(center)[B]
#place(bottom, scope: "parent", rect(height: 10pt))
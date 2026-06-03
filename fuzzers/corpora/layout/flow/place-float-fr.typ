
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 120pt, columns: 2)
#set place(float: true, clearance: 10pt)
#set rect(width: 70%)

#place(top + center, rect[I])
#place(bottom + center, scope: "parent", rect[II])

A
#v(1fr)
B
#colbreak()
C
#align(bottom)[D]
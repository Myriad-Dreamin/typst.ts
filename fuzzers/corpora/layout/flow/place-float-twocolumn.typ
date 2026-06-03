
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 100pt, columns: 2)
#set place(float: true, clearance: 10pt)
#set rect(width: 70%)

#place(top + center, scope: "parent", rect[I])
#place(top + center, rect[II])
#lines(4)
#place(top + center, rect[III])
#block(width: 100%, height: 70pt, fill: conifer)
#place(bottom + center, scope: "parent", rect[IV])
#place(bottom + center, rect[V])
#v(1pt, weak: true)
#block(width: 100%, height: 60pt, fill: aqua)
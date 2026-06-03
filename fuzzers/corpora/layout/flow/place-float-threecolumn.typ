
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 100pt, columns: 3)
#set place(float: true, clearance: 10pt)
#set rect(width: 70%)

#place(bottom + center, scope: "parent", rect[I])
#lines(21)
#place(top + center, scope: "parent", rect[II])
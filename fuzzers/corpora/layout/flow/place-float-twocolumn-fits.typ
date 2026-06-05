
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 100pt, columns: 2)
#set place(float: true, clearance: 10pt)
#set rect(width: 70%)

#lines(6)
#place(auto, scope: "parent", rect[I])
#lines(12, "1")
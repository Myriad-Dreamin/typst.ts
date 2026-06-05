
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 100pt, columns: 2)
#set place(float: true, clearance: 10pt)
#set rect(width: 70%)

#place(auto, scope: "parent", rect[I]) // Should end up `top`
#lines(4)
#place(auto, scope: "parent", rect[II])  // Should end up `bottom`
#lines(4)
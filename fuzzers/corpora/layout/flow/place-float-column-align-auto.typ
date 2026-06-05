
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 150pt, columns: 2)
#set place(auto, float: true, clearance: 10pt)
#set rect(width: 75%)

#place(rect[I])
#place(rect[II])
#place(rect[III])
#place(rect[IV])

#lines(6)

#place(rect[V])
#place(rect[VI])
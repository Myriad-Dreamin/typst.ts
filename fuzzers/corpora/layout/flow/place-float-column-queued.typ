
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 100pt, columns: 2)
#set place(float: true, clearance: 10pt)
#set rect(width: 75%)
#set text(costs: (widow: 0%, orphan: 0%))

#lines(3)

#place(top, rect[I])
#place(top, rect[II])
#place(bottom, rect[III])

#lines(3)
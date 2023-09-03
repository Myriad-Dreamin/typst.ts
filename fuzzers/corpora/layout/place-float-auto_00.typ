
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(height: 140pt)
#set place(clearance: 5pt)
#lorem(6)
#place(auto, float: true, rect[A])
#place(auto, float: true, rect[B])
#place(auto, float: true, rect[C])
#place(auto, float: true, rect[D])


#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// This bug caused an index-out-of-bounds panic when layouting paragraphs needed
// multiple reorderings.
#set page(height: 200pt)
#lines(10)

#figure(placement: auto, block(height: 100%))

#lines(3)

#lines(3)
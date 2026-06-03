
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The contents after the block should be pushed upwards.
#set page(height: 60pt)
a
#block(height: -25pt)[b]
c
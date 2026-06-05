
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that sticky blocks are still breakable.
#set page(height: 60pt)
#block(sticky: true, lines(4))
E
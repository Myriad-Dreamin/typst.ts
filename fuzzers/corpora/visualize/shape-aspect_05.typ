
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that square doesn't overflow due to its aspect ratio.
#set page(width: 40pt, height: 25pt, margin: 5pt)
#square(width: 100%)
#square(width: 100%)[Hello there]

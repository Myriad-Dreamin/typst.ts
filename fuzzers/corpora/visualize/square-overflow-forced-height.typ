
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that a height-overflowing square is laid out regardless of the
// presence of inner content.
#set page(width: 120pt, height: 60pt)
#square(height: 150%)
#square(height: 150%)[Hello there]
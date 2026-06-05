
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that a width-overflowing square is laid out regardless of the
// presence of inner content.
#set page(width: 60pt, height: 100pt)
#square(width: 150%)
#square(width: 150%)[Hello there]

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that setting a square's height beyond its default sizes it correctly.
#square()
#square(height: 60pt)
#square(width: 60pt)
#square(size: 60pt)
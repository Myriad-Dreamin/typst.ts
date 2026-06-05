
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that setting a rectangle's height beyond its default sizes it correctly.
#rect()
#rect(height: 60pt)
#rect(width: 60pt)
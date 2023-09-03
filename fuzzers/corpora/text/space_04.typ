
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that linebreak consumed surrounding spaces.
#align(center)[A \ B \ C]


#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that the pad element doesn't consume the whole region.
#set page(height: 6cm)
#align(left)[Before]
#pad(10pt, image("/assets/files/tiger.jpg"))
#align(right)[After]

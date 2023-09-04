
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test forced justification with justified break.
A B C #linebreak(justify: true)
D E F #linebreak(justify: true)

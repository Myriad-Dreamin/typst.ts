
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test calling a symbol whose field isn't an accent.
#test($pi.alt(x)$, $pi.alt/**/(x)$)